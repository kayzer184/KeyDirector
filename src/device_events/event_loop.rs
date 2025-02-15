use super::{CallbackGuard, KeyboardCallbacks};
use std::sync::{Arc, Mutex, Weak};
use std::thread::{sleep, spawn, JoinHandle};
use std::time::Duration;
use {DeviceQuery, KeyEvent};
use DeviceState;

pub(crate) struct EventLoop {
    keyboard_callbacks: Arc<KeyboardCallbacks>,
    _keyboard_thread: JoinHandle<()>,
}

fn keyboard_thread(callbacks: Weak<KeyboardCallbacks>) -> JoinHandle<()> {
    spawn(move || {
        let device_state = DeviceState::new();
        let mut prev_keys = vec![];
        while let Some(callbacks) = callbacks.upgrade() {
            let keys = device_state.get_keys();
            for key_event in &keys {
                if !prev_keys.iter().any(|prev: &KeyEvent| prev.key_code == key_event.key_code) {
                    callbacks.run_key_down(key_event);
                }
            }
            for prev_key in &prev_keys {
                if !keys.iter().any(|key| key.key_code == prev_key.key_code) {
                    let release_event = KeyEvent::new(
                        prev_key.key_code,
                        prev_key.scan_code,
                        prev_key.char,
                        false
                    );
                    callbacks.run_key_up(&release_event);
                }
            }
            prev_keys = keys;
            sleep(Duration::from_micros(100));
        }
    })
}

impl Default for EventLoop {
    fn default() -> Self {
        let keyboard_callbacks = Arc::new(KeyboardCallbacks::default());
        let _keyboard_thread = keyboard_thread(Arc::downgrade(&keyboard_callbacks));
        Self {
            keyboard_callbacks,
            _keyboard_thread,
        }
    }
}

impl EventLoop {
    pub fn on_key_down<Callback: Fn(&KeyEvent) -> bool + Send + Sync + 'static>(
        &mut self,
        callback: Callback,
    ) -> CallbackGuard<Callback> {
        let _callback = Arc::new(callback);
        self.keyboard_callbacks.push_key_down(_callback.clone());
        CallbackGuard { _callback }
    }

    pub fn on_key_up<Callback: Fn(&KeyEvent) -> bool + Send + Sync + 'static>(
        &mut self,
        callback: Callback,
    ) -> CallbackGuard<Callback> {
        let _callback = Arc::new(callback);
        self.keyboard_callbacks.push_key_up(_callback.clone());
        CallbackGuard { _callback }
    }

    pub fn on_keys<F>(&mut self, callback: F) -> CallbackGuard<F>
    where
        F: Fn(Vec<KeyEvent>) -> bool + Send + Sync + 'static,
    {
        let _callback = Arc::new(callback);
        self.keyboard_callbacks.push_keys(_callback.clone());
        CallbackGuard { _callback }
    }
}

lazy_static! {
    pub(crate) static ref EVENT_LOOP: Arc<Mutex<EventLoop>> = Default::default();
}
