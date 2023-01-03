use crate::{Side, Keys};

use super::{key::{Key, LayerCMD}, keyboard::Ninja};
use super::keyboard::KeyboardTrait;
use usbd_human_interface_device::page::Keyboard as Kc;

pub const CONF_KEY_BYTES:usize=2; //bytes per key in conf report
pub const CONF_SIZE:usize=Ninja::COLS*Ninja::ROWS*Ninja::LAYERS*Ninja::SIDES*CONF_KEY_BYTES+2;//2 byte mark size

pub fn serialize_key(key:&Key)->(u8,u8){
    match key{
        Key::Code(code)=>(0,*code as u8),
        //Key::Layer=>(1,0),
        Key::Layer(lcmd)=>
            match lcmd {
                LayerCMD::TMP(l) => (1,*l) ,
                LayerCMD::NEXT   => (2,0),
                LayerCMD::PREV   => (3,0),
                LayerCMD::FIRST  => (4,0),
                LayerCMD::LAST   => (5,0),
                LayerCMD::SET(l) => (5,*l),
            }
        ,
        Key::NoKey=>(255,0)
    }
}
pub fn deserialize_key(b1:u8,b2:u8)->Key{
    match b1{
        0=> Key::Code(Kc::from(b2)),
        1=> Key::Layer(LayerCMD::TMP(b2)),
        2=> Key::Layer(LayerCMD::NEXT),
        3=> Key::Layer(LayerCMD::PREV),
        4=> Key::Layer(LayerCMD::FIRST),
        5=> Key::Layer(LayerCMD::LAST),
        6=> Key::Layer(LayerCMD::SET(b2)),
        255=>Key::NoKey,
        _=>Key::NoKey
    }
}
pub fn serialize_keys(side:u8,layer:u8,side_data:&Side,bytes:&mut [u8;64]){
    let mut i:usize=4;
    bytes[0]=1;//keys
    bytes[1]=0;//reserved
    bytes[2]=side;
    bytes[3]=layer;
    for row in 0..Ninja::ROWS{
        for col in 0..Ninja::COLS  {
            let k=serialize_key(&side_data[row][col]);
            if i+1 < 64{
                bytes[i  ]=k.0;
                bytes[i+1]=k.1;
            }
            i+=2;
        }
    }
}
pub fn deserialize_keys(bytes:&[u8;64],keys:&mut Keys){
    let side=bytes[2] as usize;
    let layer=bytes[3] as usize;                            
    let mut k:usize=4;
    for row in 0..Ninja::ROWS{
        for col in 0..Ninja::COLS  {
            let key=deserialize_key(bytes[k],bytes[k+1]);
            if side < Ninja::SIDES && layer < Ninja::LAYERS {
                keys[side][layer][row][col]=key;
            }
            if k+1 < 64{
                k+=2;
            }
        }
    }
}