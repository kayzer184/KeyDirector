extern crate key_director;

use key_director::{DeviceQuery, DeviceState};

fn main() {
    let device_state = DeviceState::new();
    let mut prev_keys = vec![];
    loop {
        let keys = device_state.get_keys();
        if keys != prev_keys {
            println!("{:?}", keys);
        }
        prev_keys = keys;
    }
}
