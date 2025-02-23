use super::{CallbackGuard, KeyboardCallbacks};
use std::sync::{Arc, Mutex, Weak};
use std::thread::{sleep, spawn, JoinHandle};
use std::time::Duration;
use KeyEvent;
use windows::Win32::UI::WindowsAndMessaging::{CallNextHookEx, SetWindowsHookExA, WH_KEYBOARD_LL, KBDLLHOOKSTRUCT, WM_KEYDOWN, WM_SYSKEYDOWN};
use windows::Win32::Foundation::{LPARAM, WPARAM, LRESULT};
use std::os::raw::c_int;

lazy_static! {
    static ref KEYBOARD_CALLBACKS: Arc<Mutex<Option<Weak<KeyboardCallbacks>>>> = Arc::new(Mutex::new(None));
}

pub(crate) struct EventLoop {
    keyboard_callbacks: Arc<KeyboardCallbacks>,
    _keyboard_thread: JoinHandle<()>,
}

unsafe extern "system" fn keyboard_hook_proc(code: c_int, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    if code >= 0 {
        if w_param.0 as u32 == WM_KEYDOWN || w_param.0 as u32 == WM_SYSKEYDOWN {
            let kbd_struct = *(l_param.0 as *const KBDLLHOOKSTRUCT);
            
            if let Some(callbacks) = KEYBOARD_CALLBACKS.lock().unwrap().as_ref() {
                if let Some(callbacks) = callbacks.upgrade() {
                    let key_event = KeyEvent::new(
                        None,
                        kbd_struct.vkCode,
                        kbd_struct.scanCode,
                        true,
                        kbd_struct.dwExtraInfo == 1
                    );
                    
                    if !callbacks.run_key_down(&key_event) {
                        return LRESULT(1); // Блокируем клавишу
                    }
                }
            }
        }
    }
    
    CallNextHookEx(None, code, w_param, l_param)
}

fn keyboard_thread(callbacks: Weak<KeyboardCallbacks>) -> JoinHandle<()> {
    *KEYBOARD_CALLBACKS.lock().unwrap() = Some(callbacks.clone());
    
    spawn(move || {
        unsafe {
            let hook = SetWindowsHookExA(
                WH_KEYBOARD_LL,
                Some(keyboard_hook_proc),
                None,
                0
            );
            
            if hook.is_err() {
                println!("Ошибка установки хука клавиатуры");
                return;
            }
            
            // Держим поток живым
            loop {
                sleep(Duration::from_millis(50));
                if callbacks.upgrade().is_none() {
                    break;
                }
            }
        }
    })
}

impl EventLoop {
    pub fn new() -> Self {
        let keyboard_callbacks = Arc::new(KeyboardCallbacks::default());
        let keyboard_thread = keyboard_thread(Arc::downgrade(&keyboard_callbacks));
        
        Self {
            keyboard_callbacks,
            _keyboard_thread: keyboard_thread,
        }
    }

    pub fn on_key_down<Callback: Fn(&KeyEvent) -> bool + Send + Sync + 'static>(
        &mut self,
        callback: Callback,
    ) -> CallbackGuard<Callback> {
        let _callback = Arc::new(callback);
        self.keyboard_callbacks.push_key_down(_callback.clone());
        CallbackGuard { _callback }
    }

    pub fn on_key_up<Callback: Fn(&KeyEvent) -> bool + Send + Sync + 'static>(
        &mut self,
        callback: Callback,
    ) -> CallbackGuard<Callback> {
        let _callback = Arc::new(callback);
        self.keyboard_callbacks.push_key_up(_callback.clone());
        CallbackGuard { _callback }
    }

    pub fn on_keys<F>(&mut self, callback: F) -> CallbackGuard<F>
    where
        F: Fn(Vec<KeyEvent>) -> bool + Send + Sync + 'static,
    {
        let _callback = Arc::new(callback);
        self.keyboard_callbacks.push_keys(_callback.clone());
        CallbackGuard { _callback }
    }
}

lazy_static! {
    pub(crate) static ref EVENT_LOOP: Arc<Mutex<EventLoop>> = Arc::new(Mutex::new(EventLoop::new()));
}
