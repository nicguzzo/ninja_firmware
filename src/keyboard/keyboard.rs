use crate::Keys;

pub trait KeyboardTrait {
    const COLS: usize;
    const ROWS: usize;
    const LAYERS:usize=4;
    const SIDES:usize=2;
    fn get_default_keys()->Keys;    
}

pub struct Ninja;