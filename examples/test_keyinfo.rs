extern crate key_director;
use key_director::DeviceState;
use std::{thread, time::Duration, sync::Arc};

fn main() {
    println!("Запуск мониторинга клавиатуры...");
    let device_state = Arc::new(DeviceState::new());
    
    let device_state_clone = Arc::clone(&device_state);
    device_state_clone.add_callback({
        let device_state_clone = device_state_clone.clone();
        move |key| {
            println!("Нажата клавиша: {:?}", key);
            if key.is_simulated {
                return true;
            }
            if key.key_code == 69 {
                if key.is_pressed {
                    device_state_clone.press(vec![87]);
                } else {
                    device_state_clone.release(vec![87]);
                }
                return false;
            }
            if key.key_code == 83 {
                if key.is_pressed {
                    device_state_clone.press(vec![65]);
                } else {
                    device_state_clone.release(vec![65]);
                }
                return false;
            }
            if key.key_code == 68 {
                if key.is_pressed {
                    device_state_clone.press(vec![83]);
                } else {
                    device_state_clone.release(vec![83]);
                }
                return false;
            }
            if key.key_code == 70 {
                if key.is_pressed {
                    device_state_clone.press(vec![68]);
                } else {
                    device_state_clone.release(vec![68]);
                }
                return false;
            }
            if key.key_code == 65 || key.key_code == 87 || key.key_code == 83 || key.key_code == 68 {
                return false;
            }
            true
        }
    });
    
    println!("Мониторинг запущен. (Ctrl+C для выхода)");
    
    loop {
        thread::sleep(Duration::from_millis(100));
    }
}