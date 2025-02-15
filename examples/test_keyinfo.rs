extern crate key_director;
use key_director::{DeviceQuery, DeviceState};
use std::{thread, time::Duration};

fn main() {
    println!("Запуск мониторинга клавиатуры...");
    let device_state = DeviceState::new();
    
    let _guard = device_state.subscribe_keys(|keys| {
        if !keys.is_empty() {
            for key in keys {                
                if key.scan_code == 30 && key.is_pressed {
                    return false;
                }
            }
        }
        true
    });
    
    println!("Мониторинг запущен. Нажмите любую клавишу (Ctrl+C для выхода)");
    
    loop {
        thread::sleep(Duration::from_millis(10));
    }
}