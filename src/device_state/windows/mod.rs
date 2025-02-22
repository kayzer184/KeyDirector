use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, GetKeyboardState, MapVirtualKeyA,
    ToUnicodeEx, GetKeyboardLayout, MAP_VIRTUAL_KEY_TYPE, 
    VK_CAPITAL, GetKeyState
};
use windows::Win32::UI::WindowsAndMessaging::{
    GetWindowThreadProcessId, GUITHREADINFO, GetGUIThreadInfo,
    SetWindowsHookExW, UnhookWindowsHookEx, CallNextHookEx,
    WH_KEYBOARD_LL, KBDLLHOOKSTRUCT, WM_KEYDOWN, WM_SYSKEYDOWN, HHOOK,
    GetMessageW, TranslateMessage, DispatchMessageW, MSG
};
use windows::Win32::Foundation::{LPARAM, WPARAM, LRESULT, HWND};
use std::sync::{Arc, Mutex};
use std::thread;
use crate::KeyEvent;

static PREV_PRESSED: Mutex<Vec<u32>> = Mutex::new(Vec::new());
static KEYBOARD_HOOK: Mutex<Option<HHOOK>> = Mutex::new(None);
static GLOBAL_CALLBACKS: Mutex<Vec<Box<dyn Fn(&KeyEvent) -> bool + Send + Sync>>> = Mutex::new(Vec::new());

#[derive(Clone)]
pub struct DeviceState {}

impl std::fmt::Debug for DeviceState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DeviceState")
            .field("callbacks", &"<callback functions>")
            .finish()
    }
}

unsafe extern "system" fn keyboard_hook_proc(code: i32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    if code >= 0 {
        let kbd_struct = *(l_param.0 as *const KBDLLHOOKSTRUCT);
        let key_event = KeyEvent::new(
            kbd_struct.vkCode,
            kbd_struct.scanCode,
            None,
            w_param.0 as u32 == WM_KEYDOWN || w_param.0 as u32 == WM_SYSKEYDOWN
        );

        if let Ok(callbacks) = GLOBAL_CALLBACKS.lock() {
            for callback in callbacks.iter() {
                if !callback(&key_event) {
                    return LRESULT(1);
                }
            }
        }
    }
    CallNextHookEx(None, code, w_param, l_param)
}

impl DeviceState {
    pub fn new() -> DeviceState {
        thread::spawn(|| {
            unsafe {
                let hook = SetWindowsHookExW(
                    WH_KEYBOARD_LL,
                    Some(keyboard_hook_proc),
                    None,
                    0
                ).expect("Failed to set keyboard hook");
                
                *KEYBOARD_HOOK.lock().unwrap() = Some(hook);

                let mut msg = MSG::default();
                while GetMessageW(&mut msg, HWND(0), 0, 0).as_bool() {
                    TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }
            }
        });

        DeviceState {}
    }

    pub fn add_callback<F>(&self, callback: F)
    where
        F: Fn(&KeyEvent) -> bool + Send + Sync + 'static,
    {
        if let Ok(mut callbacks) = GLOBAL_CALLBACKS.lock() {
            callbacks.push(Box::new(callback));
        }
    }

    pub fn query_keymap(&self) -> Vec<KeyEvent> {
        let mut key_events = Vec::new();
        let mut keyboard_state = [0u8; 256];
        let mut current_pressed = Vec::new();
        
        unsafe {
            let mut gui_info = GUITHREADINFO {
                cbSize: std::mem::size_of::<GUITHREADINFO>() as u32,
                ..Default::default()
            };
            
            if GetGUIThreadInfo(0, &mut gui_info).as_bool() {
                let mut process_id = 0u32;
                let thread_id = GetWindowThreadProcessId(gui_info.hwndFocus, Some(&mut process_id));
                let keyboard_layout = GetKeyboardLayout(thread_id);
                
                if GetKeyboardState(&mut keyboard_state).as_bool() {
                    let caps_on = (GetKeyState(VK_CAPITAL.0 as i32) & 1) != 0;
                    if caps_on {
                        keyboard_state[VK_CAPITAL.0 as usize] |= 1;
                    }

                    for key_code in 0..256 {
                        let state = GetAsyncKeyState(key_code);
                        let is_pressed = (state as u32 & 0x8000) != 0;
                        let was_pressed = (state as u32 & 0x0001) != 0;
                        
                        if is_pressed {
                            current_pressed.push(key_code as u32);
                        }

                        if was_pressed || (PREV_PRESSED.lock().unwrap().contains(&(key_code as u32)) && !is_pressed) {
                            let scan_code = MapVirtualKeyA(key_code as u32, MAP_VIRTUAL_KEY_TYPE(0));
                            let mut buff = [0u16; 8];
                            
                            let chars = ToUnicodeEx(
                                key_code as u32,
                                scan_code,
                                &keyboard_state,
                                &mut buff,
                                8,
                                keyboard_layout
                            );
                            
                            let character = if chars > 0 {
                                std::char::from_u32(buff[0] as u32)
                            } else {
                                None
                            };

                            key_events.push(KeyEvent::new(
                                key_code as u32,
                                scan_code,
                                character,
                                is_pressed
                            ));
                        }
                    }
                }
            }
            
            if let Ok(mut prev_pressed) = PREV_PRESSED.lock() {
                *prev_pressed = current_pressed;
            }
        }
        
        key_events
    }
}

impl Drop for DeviceState {
    fn drop(&mut self) {
        if let Ok(mut hook) = KEYBOARD_HOOK.lock() {
            if let Some(h) = hook.take() {
                unsafe {
                    UnhookWindowsHookEx(h);
                }
            }
        }
    }
}

lazy_static! {
    static ref GLOBAL_DEVICE_STATE: Arc<Mutex<DeviceState>> = Arc::new(Mutex::new(DeviceState {}));
}
