extern crate key_director;
use key_director::DeviceState;
use std::{thread, time::Duration};

fn main() {
    println!("Запуск мониторинга клавиатуры...");
    let device_state = DeviceState::new();
    
    device_state.add_callback(|key| {
        if key.scan_code == 30 {
            return false;
        }
        if key.scan_code == 31 {
            return false;
        }
        true
    });
    
    println!("Мониторинг запущен. Попробуйте нажать клавиши A или S (Ctrl+C для выхода)");
    
    loop {
        thread::sleep(Duration::from_millis(100));
    }
}