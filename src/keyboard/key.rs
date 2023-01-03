use usbd_human_interface_device::page::Keyboard as Kc;

#[derive(Clone,Copy)]
pub enum Key{
    Code(Kc),
    Layer,
    NoKey,
}

impl defmt::Format for Key {
    fn format(&self, f: defmt::Formatter) {
        match self{
            Key::Code(k)=>{
                defmt::write!(f,"k {}",*k as u8)
            },
            Key::Layer=>{
                defmt::write!(f,"Lyr")
            },
            Key::NoKey=>{
                defmt::write!(f,"Nk")
            }
        }
    }
}