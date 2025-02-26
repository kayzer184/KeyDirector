use macos_accessibility_client::accessibility::application_is_trusted_with_prompt;
use cocoa::base::{id, nil};
use cocoa::foundation::NSAutoreleasePool;
use core_graphics::event::{CGEvent, CGEventFlags, CGEventType, CGKeyCode, CGEventTap, CGEventTapLocation, CGEventMask, CGEventTapPlacement, CGEventTapOptions};
use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use crate::KeyEvent;
use core_foundation::mach_port::CFMachPort;
use core_foundation::runloop::{CFRunLoop, kCFRunLoopCommonModes};
use std::cell::RefCell;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

const K_CG_KEYBOARD_EVENT_KEYCODE: u32 = 9;  // Core Graphics keyboard event keycode constant

lazy_static! {
    static ref GLOBAL_CALLBACKS: Mutex<Vec<Box<dyn Fn(&KeyEvent) -> bool + Send + Sync>>> = Mutex::new(Vec::new());
    static ref CURRENT_KEYS: Mutex<HashMap<u32, KeyEvent>> = Mutex::new(HashMap::new());
    static ref SIMULATION_FLAG: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));
}

thread_local! {
    static EVENT_TAP: RefCell<Option<CGEventTap<'static>>> = RefCell::new(None);
}

pub struct DeviceState {
    initialized: Arc<AtomicBool>,
}

impl DeviceState {
    pub fn new() -> Self {
        if !application_is_trusted_with_prompt() {
            panic!("Application needs Accessibility permissions to monitor keyboard events");
        }

        let initialized = Arc::new(AtomicBool::new(false));
        let init_clone = initialized.clone();
        
        thread::spawn(move || {
            println!("Starting event tap thread");
            let pool = unsafe { NSAutoreleasePool::new(nil) };
            
            EVENT_TAP.with(|tap| {
                println!("Creating event tap");
                let event_tap = CGEventTap::new(
                    CGEventTapLocation::HID,
                    CGEventTapPlacement::HeadInsertEventTap,
                    CGEventTapOptions::Default,
                    vec![CGEventType::KeyDown, CGEventType::KeyUp],
                    move |proxy, event_type, event| unsafe {
                        println!("Received event: {:?}", event_type);
                        if let Some(key_event) = handle_keyboard_event(event_type, event) {
                            println!("Processed key event: {:?}", key_event);
                            if let Ok(mut current_keys) = CURRENT_KEYS.lock() {
                                if key_event.is_pressed {
                                    current_keys.insert(key_event.key_code, key_event.clone());
                                } else {
                                    current_keys.remove(&key_event.key_code);
                                }
                            }
                            
                            if let Ok(callbacks) = GLOBAL_CALLBACKS.lock() {
                                for callback in callbacks.iter() {
                                    if !callback(&key_event) {
                                        return None;
                                    }
                                }
                            }
                        }
                        Some(event.clone())
                    },
                ).expect("Failed to create event tap");
                
                println!("Enabling event tap");
                event_tap.enable();
                
                // Get the run loop and add the event tap to it
                let run_loop = CFRunLoop::get_current();
                let tap_port = &event_tap.mach_port;
                let run_loop_source = CFMachPort::create_runloop_source(tap_port, 0).expect("Failed to create run loop source");
                unsafe { run_loop.add_source(&run_loop_source, kCFRunLoopCommonModes) };
                
                *tap.borrow_mut() = Some(event_tap);
            });

            init_clone.store(true, Ordering::SeqCst);
            println!("Starting run loop");
            CFRunLoop::run_current();
            unsafe { pool.drain(); }
        });
        
        // Wait for initialization
        while !initialized.load(Ordering::SeqCst) {
            thread::sleep(Duration::from_millis(10));
        }
        
        DeviceState { initialized }
    }

    /// returns `None` if app doesn't accessibility permissions.
    pub fn checked_new() -> Option<DeviceState> {
        if has_accessibility() {
            Some(DeviceState::new())
        } else {
            None
        }
    }

    pub fn get_keys(&self) -> Vec<KeyEvent> {
        if let Ok(current_keys) = CURRENT_KEYS.lock() {
            current_keys.values().cloned().collect()
        } else {
            Vec::new()
        }
    }

    pub fn add_callback<F>(&self, callback: F)
    where
        F: Fn(&KeyEvent) -> bool + Send + Sync + 'static,
    {
        if let Ok(mut callbacks) = GLOBAL_CALLBACKS.lock() {
            callbacks.push(Box::new(callback));
        }
    }

    pub fn press(&self, keys: Vec<u32>) {
        let source = CGEventSource::new(CGEventSourceStateID::Private)
            .expect("Failed to create event source");
            
        for key in keys {
            let down_event = CGEvent::new_keyboard_event(
                source.clone(),
                key as CGKeyCode,
                true
            ).expect("Failed to create key down event");
            
            down_event.set_flags(CGEventFlags::empty());
            down_event.post(CGEventTapLocation::HID);
        }
    }

    pub fn release(&self, keys: Vec<u32>) {
        let source = CGEventSource::new(CGEventSourceStateID::Private)
            .expect("Failed to create event source");
            
        for key in keys {
            let up_event = CGEvent::new_keyboard_event(
                source.clone(),
                key as CGKeyCode,
                false
            ).expect("Failed to create key up event");
            
            up_event.set_flags(CGEventFlags::empty());
            up_event.post(CGEventTapLocation::HID);
        }
    }
}

impl Drop for DeviceState {
    fn drop(&mut self) {
        EVENT_TAP.with(|tap| {
            if let Some(tap) = tap.borrow_mut().take() {
                tap.enable();
            }
        });
    }
}

unsafe extern "C" fn event_callback(_proxy: *const std::ffi::c_void, event_type: CGEventType, event: &CGEvent) -> Option<CGEvent> {
    if let Some(key_event) = handle_keyboard_event(event_type, event) {
        if let Ok(mut current_keys) = CURRENT_KEYS.lock() {
            if key_event.is_pressed {
                current_keys.insert(key_event.key_code, key_event.clone());
            } else {
                current_keys.remove(&key_event.key_code);
            }
        }
        
        if let Ok(callbacks) = GLOBAL_CALLBACKS.lock() {
            for callback in callbacks.iter() {
                if !callback(&key_event) {
                    return None;
                }
            }
        }
    }
    Some(event.clone())
}

fn handle_keyboard_event(event_type: CGEventType, event: &CGEvent) -> Option<KeyEvent> {
    match event_type {
        CGEventType::KeyDown | CGEventType::KeyUp => {
            let key_code = event.get_integer_value_field(K_CG_KEYBOARD_EVENT_KEYCODE) as u32;
            let flags = event.get_flags();
            
            Some(KeyEvent::new(
                None, // TODO: Implement character conversion
                key_code,
                key_code, // Using keycode as scancode for now
                matches!(event_type, CGEventType::KeyDown),
                flags.contains(CGEventFlags::CGEventFlagNull)
            ))
        }
        _ => None
    }
}

/// Returns true if the Accessibility permissions necessary for this library to work are granted
/// to this process
///
/// If this returns false, the app can request them through the OS APIs, or the user can:
///   1. open the MacOS system preferences
///   2. go to Security -> Privacy
///   3. scroll down to Accessibility and unlock it
///   4. Add the app that is using device_query (such as your terminal) to the list
///
fn has_accessibility() -> bool {
    application_is_trusted_with_prompt()
}
