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
            //info!("i2c read/write error")
        }
    }
    //info!("m{}",ninja_kb.matrices[Ninja::MAIN]);
    //info!("s{}",ninja_kb.matrices[Ninja::SECONDARY][0]);
    for side in 0..Ninja::SIDES{
        let side2=Ninja::SIDES_A[side];
        for col in 0..Ninja::COLS  {
            for row in 0..Ninja::ROWS{
                let index=row*Ninja::COLS+col;
                let byte=index>>3;
                let bit=(index%8) as u8;
                //pressed
                let mat1=ninja_kb.matrices[side2][0][byte] & (1<<bit);
                let mat2=ninja_kb.matrices[side2][1][byte] & (1<<bit);
                if mat1!=0 && mat2==0{
                    ninja_kb.leds[4].set_low();
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
                                ninja_kb.events[side][row][col]=(code,k as u8);
                                //ninja_kb.report_buff_layer[k]=(ninja_kb.layer as u8,row as u8,col as u8);
                            }
                        }
                        _ =>()
                    }                        
                }
                //released
                if mat1==0 && mat2!=0{
                    ninja_kb.leds[4].set_high();
                    //go through all layers to release previus keys
                    //for layer in 0..Ninja::LAYERS{
                      let layer=ninja_kb.layer;

                      let code=ninja_kb.events[side][row][col];
                      if code.0 != Kc::NoEventIndicated && (code.1 as usize) < REPORT_BUFF_MAX {
                        event=true;
                        ninja_kb.report_buff[code.1 as usize]=Kc::NoEventIndicated;
                        break;
                      }else{

                        match ninja_kb.keys[side][layer][row][col]{
                            Key::Layer(lcmd)=>{
                                info!("released LayerCmd {}",lcmd);
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
                                    }/*else if ninja_kb.report_buff_layer[i].1==row as u8 &&
                                            ninja_kb.report_buff_layer[i].2==col as u8 {
                                        event=true;
                                        ninja_kb.report_buff[i]=Kc::NoEventIndicated;
                                        ninja_kb.report_buff_layer[i]=(0,255,255);
                                        break;
                                    }*/
                                }
                            }
                            _ =>()
                        }
                      }
                    //}
                }
            }
        }
    }
    for l in 0..4{
      ninja_kb.leds[l].set_low();
    }
    ninja_kb.leds[ninja_kb.layer].set_high();
    
    event
}