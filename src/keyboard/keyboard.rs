use usbd_human_interface_device::page::Keyboard as Kc;
use crate::{Keys, NinjaKb, secondary_side::SecondarySideI2C, I2cProxy, REPORT_BUFF_MAX};
use defmt_rtt as _; 
use defmt::{info};

use super::key::{LayerCMD, Key};
pub trait KeyboardTrait {
    const MODEL:u8;
    const COLS: usize;
    const ROWS: usize;
    const MAIN: usize;
    const SECONDARY: usize;
    const LAYERS:usize=4;
    const SIDES:usize=2;
    const SIDES_A:[usize;2]=[Self::MAIN,Self::SECONDARY];
    fn get_default_keys()->Keys;
}
pub const KB_N_BYTES:usize = ((Ninja::COLS*Ninja::ROWS) + 7 & !7)/8;
pub struct Ninja;

pub fn update_kb_state(ninja_kb:&mut NinjaKb ,secondary_side:&mut SecondarySideI2C<I2cProxy>)-> bool{
    let mut event=false;
    for byte in 0..KB_N_BYTES  {
        ninja_kb.matrices[Ninja::MAIN]     [1][byte]=ninja_kb.matrices[Ninja::MAIN]     [0][byte];
        ninja_kb.matrices[Ninja::SECONDARY][1][byte]=ninja_kb.matrices[Ninja::SECONDARY][0][byte];
    }
    for col in 0..Ninja::COLS  {
        ninja_kb.cols[col].set_low();
        for row in 0..Ninja::ROWS {
            let index=row*Ninja::COLS+col;
            let byte=index>>3;
            let bit=(index%8) as u8;
            if ninja_kb.rows[row].is_low(){
                ninja_kb.matrices[Ninja::MAIN][0][byte]|=1<<bit;
            }else{
                ninja_kb.matrices[Ninja::MAIN][0][byte]&= !(1<<bit);
            }
        }
        ninja_kb.cols[col].set_high();
    }
    match secondary_side.read_keys(){
        Ok(buffer) =>{
            ninja_kb.matrices[Ninja::SECONDARY][0]=buffer;
        },
        Err(_) =>{
            info!("i2c read/write error")
        }
    }
    //info!("m{}",ninja_kb.matrices[Ninja::MAIN]);
    //info!("s{}",ninja_kb.matrices[Ninja::SECONDARY]);
    for side in 0..Ninja::SIDES{
        let side2=Ninja::SIDES_A[side];
        for col in 0..Ninja::COLS  {
            for row in 0..Ninja::ROWS{
                let index=row*Ninja::COLS+col;
                let byte=index>>3;
                let bit=(index%8) as u8;
                //pressed
                let m1=ninja_kb.matrices[side2][0][byte] & (1<<bit);
                let m2=ninja_kb.matrices[side2][1][byte] & (1<<bit);
                if m1!=0 && m2==0{
                    ninja_kb.led.set_low();
                    match ninja_kb.keys[side][ninja_kb.layer][row][col]{
                        Key::Layer(lcmd)=>{
                            info!("pressed LayerCmd {}",lcmd);
                            match lcmd{
                                LayerCMD::TMP(l) => {
                                    let l=l as usize;
                                    if l>=0 && l<Ninja::LAYERS {
                                        ninja_kb.last_layer=ninja_kb.layer;
                                        ninja_kb.layer=l as usize;
                                    }
                                },
                                LayerCMD::NEXT => {
                                    if ninja_kb.layer < Ninja::LAYERS {
                                       ninja_kb.layer+=1;
                                    }
                                },
                                LayerCMD::PREV   => {
                                    if ninja_kb.layer >0 {
                                       ninja_kb.layer-=1;
                                    }
                                },
                                LayerCMD::FIRST  => {
                                    ninja_kb.layer=0;
                                },
                                LayerCMD::LAST   => {
                                    ninja_kb.layer=Ninja::LAYERS-1;
                                },
                                LayerCMD::SET(l) => {
                                    let l=l as usize;
                                    if l>=0 && l<Ninja::LAYERS {
                                        ninja_kb.layer=l;
                                    }
                                },
                            }
                            event=false;
                            continue;
                        },
                        Key::Code(code)=>{
                            info!("pressed {}",code as u8);
                            let mut k=REPORT_BUFF_MAX;
                            let mut duplicate=false;
                            for i in 0..REPORT_BUFF_MAX{
                                if k==REPORT_BUFF_MAX && ninja_kb.report_buff[i]==Kc::NoEventIndicated {
                                    k=i;
                                }
                                if ninja_kb.report_buff[i]==code{
                                    duplicate=true;
                                    break;
                                }
                            }
                            if !duplicate && k< REPORT_BUFF_MAX {
                                event=true;
                                ninja_kb.report_buff[k]=code;
                            }
                        }
                        _ =>()
                    }                        
                }
                //released
                if m1==0 && m2!=0{
                    ninja_kb.led.set_high();
                    match ninja_kb.keys[side][ninja_kb.layer][row][col]{
                        Key::Layer(lcmd)=>{
                            match lcmd{
                                LayerCMD::TMP(_l) => {
                                    ninja_kb.layer=ninja_kb.last_layer;
                                },
                                _ => (),
                            }
                            event=false;
                            continue;
                        },
                        Key::Code(code)=>{
                            info!("released {}",code as u8);
                            for i in 0..REPORT_BUFF_MAX{
                                if ninja_kb.report_buff[i]==code{
                                    event=true;
                                    ninja_kb.report_buff[i]=Kc::NoEventIndicated;
                                    break;
                                }
                            }
                        }
                        _ =>()
                    }
                }
            }
        }
    }
    event
}