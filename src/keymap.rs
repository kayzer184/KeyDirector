use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KeyEvent {
    pub char: Option<char>,
    pub key_code: u32,
    pub scan_code: u32,
    pub is_pressed: bool
}
impl KeyEvent {
    pub fn new(key_code: u32, scan_code: u32, character: Option<char>, is_pressed: bool) -> Self {
        KeyEvent {
            char: character,
            key_code,
            scan_code,
            is_pressed
        }
    }
}
