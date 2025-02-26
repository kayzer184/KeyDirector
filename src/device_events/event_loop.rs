use super::{CallbackGuard, KeyboardCallbacks};
use std::sync::{Arc, Mutex, Weak};
use std::thread::{sleep, spawn, JoinHandle};
use std::time::Duration;
use KeyEvent;

pub(crate) struct EventLoop {
    keyboard_callbacks: Arc<KeyboardCallbacks>,
    _keyboard_thread: JoinHandle<()>,
}

fn keyboard_thread(callbacks: Weak<KeyboardCallbacks>) -> JoinHandle<()> {
    spawn(move || {
        loop {
            sleep(Duration::from_millis(50));
            if callbacks.upgrade().is_none() {
                break;
            }
        }
    })
}

impl EventLoop {
    pub fn new() -> Self {
        let keyboard_callbacks = Arc::new(KeyboardCallbacks::default());
        let keyboard_thread = keyboard_thread(Arc::downgrade(&keyboard_callbacks));
        
        Self {
            keyboard_callbacks,
            _keyboard_thread: keyboard_thread,
        }
    }

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
    pub(crate) static ref EVENT_LOOP: Arc<Mutex<EventLoop>> = Arc::new(Mutex::new(EventLoop::new()));
}
