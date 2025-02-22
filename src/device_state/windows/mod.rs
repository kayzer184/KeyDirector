use windows::Win32::UI::WindowsAndMessaging::{
    SetWindowsHookExW, UnhookWindowsHookEx, CallNextHookEx,
    WH_KEYBOARD_LL, KBDLLHOOKSTRUCT, WM_KEYDOWN, WM_SYSKEYDOWN, HHOOK,
    GetMessageW, TranslateMessage, DispatchMessageW, MSG, GUITHREADINFO,
    GetGUIThreadInfo, GetWindowThreadProcessId
};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetKeyboardLayout, GetKeyboardState, VK_CAPITAL, GetKeyState,
    ToUnicodeEx
};
use windows::Win32::Foundation::{LPARAM, WPARAM, LRESULT, HWND};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use crate::KeyEvent;
use lazy_static;
use std::thread;

lazy_static! {
    static ref GLOBAL_CALLBACKS: Mutex<Vec<Box<dyn Fn(&KeyEvent) -> bool + Send + Sync>>> = Mutex::new(Vec::new());
    static ref CURRENT_KEYS: Mutex<HashMap<u32, KeyEvent>> = Mutex::new(HashMap::new());
}

static KEYBOARD_HOOK: Mutex<Option<HHOOK>> = Mutex::new(None);

#[derive(Clone)]
pub struct DeviceState {}

unsafe extern "system" fn keyboard_hook_proc(code: i32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    if code >= 0 {
        let kbd_struct = *(l_param.0 as *const KBDLLHOOKSTRUCT);
        let is_pressed = w_param.0 as u32 == WM_KEYDOWN || w_param.0 as u32 == WM_SYSKEYDOWN;
        
        let mut keyboard_state = [0u8; 256];
        GetKeyboardState(&mut keyboard_state);
        
        let caps_on = (GetKeyState(VK_CAPITAL.0 as i32) & 1) != 0;
        if caps_on {
            keyboard_state[VK_CAPITAL.0 as usize] |= 1;
        }
        
        let mut gui_info = GUITHREADINFO {
            cbSize: std::mem::size_of::<GUITHREADINFO>() as u32,
            ..Default::default()
        };
        
        let character = if GetGUIThreadInfo(0, &mut gui_info).as_bool() {
            let thread_id = GetWindowThreadProcessId(gui_info.hwndFocus, None);
            let keyboard_layout = GetKeyboardLayout(thread_id);
            
            let mut buff = [0u16; 8];
            let chars = ToUnicodeEx(
                kbd_struct.vkCode,
                kbd_struct.scanCode,
                &keyboard_state,
                &mut buff,
                8,
                keyboard_layout
            );
            
            if chars > 0 {
                std::char::from_u32(buff[0] as u32)
            } else {
                None
            }
        } else {
            None
        };

        let key_event = KeyEvent::new(
            character,
            kbd_struct.vkCode,
            kbd_struct.scanCode,
            is_pressed
        );

        // Обновляем состояние клавиш
        if let Ok(mut current_keys) = CURRENT_KEYS.lock() {
            if is_pressed {
                current_keys.insert(kbd_struct.vkCode, key_event.clone());
            } else {
                current_keys.remove(&kbd_struct.vkCode);
            }
        }

        // Проверяем callbacks для блокировки
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
