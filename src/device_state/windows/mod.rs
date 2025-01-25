use windows::Win32::Foundation::POINT;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, GetKeyboardState, MapVirtualKeyA, ToUnicodeEx,
    GetKeyboardLayout, VK_LBUTTON, VK_RBUTTON, VK_MBUTTON, 
    VK_XBUTTON1, VK_XBUTTON2, MAP_VIRTUAL_KEY_TYPE, VK_CAPITAL,
    GetKeyState
};
use windows::Win32::UI::WindowsAndMessaging::{
    GetCursorPos, GetForegroundWindow, GetWindowThreadProcessId
};
use windows::Win32::System::Threading::{
    GetCurrentThreadId, AttachThreadInput
};
use std::sync::Mutex;
use crate::KeyEvent;
use crate::MouseState;

static PREV_PRESSED: Mutex<Vec<u32>> = Mutex::new(Vec::new());

#[derive(Debug, Clone)]
pub struct DeviceState;

impl DeviceState {
    pub fn new() -> DeviceState {
        DeviceState {}
    }

    pub fn query_keymap(&self) -> Vec<KeyEvent> {
        let mut key_events = Vec::new();
        let mut keyboard_state = [0u8; 256];
        let mut current_pressed = Vec::new();
        
        unsafe {
            let foreground_window = GetForegroundWindow();
            let mut process_id = 0u32;
            let window_thread = GetWindowThreadProcessId(foreground_window, Some(&mut process_id as *mut u32));
            let current_thread = GetCurrentThreadId();
            
            AttachThreadInput(current_thread, window_thread, true);
            
            let keyboard_layout = GetKeyboardLayout(window_thread);
            
            let caps_on = (GetKeyState(VK_CAPITAL.0 as i32) & 1) != 0;
            GetKeyboardState(&mut keyboard_state);
            
            if caps_on {
                keyboard_state[VK_CAPITAL.0 as usize] |= 1;
            }

            AttachThreadInput(current_thread, window_thread, false);

            for key_code in 0..256 {
                let state = GetAsyncKeyState(key_code);
                let is_pressed = (state as u32 & 0x8000) != 0;
                let was_pressed = (state as u32 & 0x0001) != 0;
                
                if is_pressed {
                    current_pressed.push(key_code as u32);
                }

                if was_pressed || (PREV_PRESSED.lock().unwrap().contains(&(key_code as u32)) && !is_pressed) {
                    let scan_code = MapVirtualKeyA(key_code as u32, MAP_VIRTUAL_KEY_TYPE(0));
                    let mut buff = [0u16; 2];
                    
                    let chars = ToUnicodeEx(
                        key_code as u32,
                        scan_code,
                        &keyboard_state,
                        &mut buff,
                        2,
                        keyboard_layout
                    );
                    
                    let character = if chars > 0 {
                        std::char::from_u32(buff[0] as u32)
                    } else if chars == -1 {
                        None
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
            
            *PREV_PRESSED.lock().unwrap() = current_pressed;
        }
        
        key_events
    }

    pub fn query_pointer(&self) -> MouseState {
        let point = &mut POINT { x: 0, y: 0 };
        let button1pressed;
        let button2pressed;
        let button3pressed;
        let button4pressed;
        let button5pressed;
        let coords;
        
        unsafe {
            coords = if GetCursorPos(point).into() {
                (point.x, point.y)
            } else {
                (0, 0)
            };
            
            button1pressed = GetAsyncKeyState(VK_LBUTTON.0 as i32) as u32 & 0x8000 != 0;
            button2pressed = GetAsyncKeyState(VK_RBUTTON.0 as i32) as u32 & 0x8000 != 0;
            button3pressed = GetAsyncKeyState(VK_MBUTTON.0 as i32) as u32 & 0x8000 != 0;
            button4pressed = GetAsyncKeyState(VK_XBUTTON1.0 as i32) as u32 & 0x8000 != 0;
            button5pressed = GetAsyncKeyState(VK_XBUTTON2.0 as i32) as u32 & 0x8000 != 0;
        }

        MouseState {
            coords,
            button_pressed: vec![
                false,
                button1pressed,
                button2pressed,
                button3pressed,
                button4pressed,
                button5pressed,
            ],
        }
    }
}
