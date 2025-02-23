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
            if key.is_pressed && key.key_code == 8 {
                device_state_clone.simulate(vec![65]);
                return false;
            }
            if key.is_pressed && key.key_code == 66 {
                device_state_clone.simulate(vec![72]);
                return false;
            }
            if key.is_pressed && key.key_code == 65 {
                device_state_clone.simulate(vec![8]);
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