use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KeyEvent {
    pub char: Option<char>,
    pub key_code: u32,
    pub scan_code: u32,
    pub is_pressed: bool,
    pub is_simulated: bool
}

impl KeyEvent {
    pub fn new(character: Option<char>, key_code: u32, scan_code: u32, is_pressed: bool, is_simulated: bool) -> Self {
        KeyEvent {
            char: character,
            key_code,
            scan_code,
            is_pressed,
            is_simulated
        }
    }
}
