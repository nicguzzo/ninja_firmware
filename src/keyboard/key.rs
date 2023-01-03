use usbd_human_interface_device::page::Keyboard as Kc;

#[derive(Clone,Copy)]
pub enum LayerCMD{
    TMP(u8), //temporary layer, with layer number
    NEXT, //
    PREV,
    FIRST,
    LAST,
    SET(u8) //set specific layer
}

#[derive(Clone,Copy)]
pub enum Key{
    Code(Kc),
    Layer(LayerCMD),
    NoKey,
}

impl defmt::Format for Key {
    fn format(&self, f: defmt::Formatter) {
        match self{
            Key::Code(k)=>{
                defmt::write!(f,"kycode {}",*k as u8)
            },
            Key::Layer(c)=>{
                defmt::write!(f,"LayerCmd {}",*c)
            },
            Key::NoKey=>{
                defmt::write!(f,"Nokey")
            }
        }
    }
}

impl defmt::Format for LayerCMD {
    fn format(&self, f: defmt::Formatter) {
        match self{
            LayerCMD::TMP(l) => defmt::write!(f,"Tmp layer: {}",l),
            LayerCMD::NEXT   => defmt::write!(f,"Next"),
            LayerCMD::PREV   => defmt::write!(f,"Prev"),
            LayerCMD::FIRST  => defmt::write!(f,"First"),
            LayerCMD::LAST   => defmt::write!(f,"Last"),
            LayerCMD::SET(l) => defmt::write!(f,"Set layer: {}",l),
        }
    }
}