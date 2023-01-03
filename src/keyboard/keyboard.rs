use crate::Keys;

pub trait KeyboardTrait {
    const COLS: usize;
    const ROWS: usize;
    const MAIN: usize;
    const SECONDARY: usize;
    const LAYERS:usize=4;
    const SIDES:usize=2;
    fn get_default_keys()->Keys;
}
pub const KB_N_BYTES:usize = ((Ninja::COLS*Ninja::ROWS) + 7 & !7)/8;
pub struct Ninja;