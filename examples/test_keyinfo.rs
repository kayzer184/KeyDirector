extern crate key_director;
use key_director::{DeviceQuery, DeviceState};
use std::{thread, time::Duration};

fn main() {
    println!("Запуск мониторинга клавиатуры...");
    let device_state = DeviceState::new();
    
    let _guard = device_state.subscribe_keys(|keys| {
        if !keys.is_empty() {
            for key in keys {
                println!("Нажата клавиша: {:?}", key);
                println!("Код клавиши: {}", key.key_code);
                println!("Символ: {:?}", key.char);
                println!("Скан-код: {}", key.scan_code);
                println!("Нажата: {}", key.is_pressed);
                println!("---");
            }
        }
    });
    
    println!("Мониторинг запущен. Нажмите любую клавишу (Ctrl+C для выхода)");
    
    loop {
        thread::sleep(Duration::from_millis(100));
    }
}