use std::sync::{Arc, Mutex, Weak};
use KeyEvent;

/// Keyboard callback.
pub type KeyboardCallback = dyn Fn(&KeyEvent) -> bool + Sync + Send + 'static;

/// Keys callback.
pub type KeysCallback = dyn Fn(Vec<KeyEvent>) -> bool + Sync + Send + 'static;

/// Keyboard callbacks.
#[derive(Default)]
pub(crate) struct KeyboardCallbacks {
    key_down: Mutex<Vec<Weak<KeyboardCallback>>>,
    key_up: Mutex<Vec<Weak<KeyboardCallback>>>,
    keys: Mutex<Vec<Weak<KeysCallback>>>,
}

impl KeyboardCallbacks {
    pub fn push_key_up(&self, callback: Arc<KeyboardCallback>) {
        if let Ok(mut key_up) = self.key_up.lock() {
            let callback = Arc::downgrade(&callback);
            key_up.push(callback)
        }
    }

    pub fn push_key_down(&self, callback: Arc<KeyboardCallback>) {
        if let Ok(mut key_down) = self.key_down.lock() {
            let callback = Arc::downgrade(&callback);
            key_down.push(callback)
        }
    }

    pub fn push_keys(&self, callback: Arc<KeysCallback>) {
        if let Ok(mut keys) = self.keys.lock() {
            let callback = Arc::downgrade(&callback);
            keys.push(callback)
        }
    }
}
