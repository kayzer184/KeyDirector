extern crate key_director;

use key_director::{DeviceQuery, DeviceState};
use std::{thread, time::Duration};

fn main() {
    let device_state = DeviceState::new();
    
    loop {
        let keys = device_state.get_keys();
        if !keys.is_empty() {
            for key in &keys {
                let char_repr = match key.char {
                    Some(c) => {
                        if c.is_control() {
                            format!("0x{:X}", key.key_code)
                        } else {
                            format!("\"{}\"", c)
                        }
                    },
                    None => format!("0x{:X}", key.key_code)
                };
                
                println!("{{ char: {}, keyCode: {}, scanCode: {}, isPressed: {} }}", 
                    char_repr,
                    key.key_code,
                    key.scan_code,
                    key.is_pressed
                );
            }
            println!("---");
        }
        thread::sleep(Duration::from_millis(10));
    }
} 