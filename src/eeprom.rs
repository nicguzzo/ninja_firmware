
use eeprom24x::{Eeprom24x, SlaveAddr};
use defmt::{info};

use crate::keyboard::keyboard::Ninja;
use crate::{I2cProxy, Keys};
use crate::keyboard::conf::{CONF_SIZE, serialize_key, deserialize_key};

use crate::keyboard::keyboard::KeyboardTrait;

pub type EepromT=Eeprom24x<I2cProxy,eeprom24x::page_size::B32,eeprom24x::addr_size::TwoBytes>;

const PAGE_SIZE:usize=32;
const CONF_PAGES:usize=(CONF_SIZE>>5)+1;
const CONF_PAGES_SIZE:usize=PAGE_SIZE*CONF_PAGES;
const EEPROM_MARK:u8 = 0xAB;

pub fn new_eeprom(proxy:I2cProxy)->EepromT{
    let address = SlaveAddr::default();
    return  Eeprom24x::new_24x32(proxy, address);
    
}

pub fn read_all(keys:&mut Keys,eeprom:&mut EepromT,delay_eeprom_cycles:u32){
    info!("read conf from eeprom");
    let memory_address = 0;
    match eeprom.read_byte(memory_address){
        Ok(_)=>{
            info!("eeprom read_byte");
            let read_data = eeprom.read_byte(memory_address).unwrap();        
            info!("mark {}", read_data);
            if read_data!=EEPROM_MARK{
                write_conf_to_eeprom(keys,eeprom,delay_eeprom_cycles);
            }else{
                info!("reading keys");
                read_conf_from_eeprom(keys,eeprom);
                info!("keys {}",keys);
            }
        },
        Err(_)=>info!("eeprom read_byte error")
    }
    info!("read conf from eeprom done.");
}
pub fn reset(eeprom:&mut EepromT){
    let memory_address = 0;
    eeprom.write_byte(memory_address,0).unwrap();
}

pub fn write_conf_to_eeprom(keys:&mut Keys,eeprom:&mut EepromT,delay:u32){
    let mut bytes=[[0u8;PAGE_SIZE];CONF_PAGES];
    let mut i:usize=2;
    let mut p=0;
    bytes[p][0]=EEPROM_MARK;
    info!("writing.");
    for side in 0..Ninja::SIDES{
        for layer in 0..Ninja::LAYERS{
            for col in 0..Ninja::COLS{
                for row in 0..Ninja::ROWS{
                    let k=serialize_key(&keys[side][layer][row][col]);
                    if i>=32{
                        i=0;
                        p+=1;
                    }
                    bytes[p][i  ]=k.0;
                    bytes[p][i+1]=k.1;
                    i+=2;
                }
            }
        }
    }
    let memory_address = 0u32;
    for page in 0..CONF_PAGES{
        match eeprom.write_page(memory_address+((page as u32) << 5), &bytes[page]){
            Ok(_)=>info!("eeprom page {} written",page),
            Err(eeprom24x::Error::TooMuchData)=> info!("Error::TooMuchData"),
            Err(_)=>info!("eeprom write error")
        }
        cortex_m::asm::delay(delay);
    }
}

pub fn read_conf_from_eeprom(keys:&mut Keys,eeprom:&mut EepromT){
    let memory_address = 0;
    let mut bytes=[0u8;CONF_PAGES_SIZE];
    info!("reading.");
    match eeprom.read_data(memory_address, &mut bytes){
        Ok(_)=>{            
            let mut i:usize=2;
            if bytes[0]==EEPROM_MARK{
                for side in 0..Ninja::SIDES{
                    for layer in 0..Ninja::LAYERS{
                        for col in 0..Ninja::COLS{
                            for row in 0..Ninja::ROWS{
                                keys[side][layer][row][col]=deserialize_key(bytes[i],bytes[i+1]);
                                i+=2;
                            }
                        }
                    }
                }
            }else{
                info!("no mark found");
            }
            info!("reading done.");
        },
        Err(_)=>info!("eeprom read error")
    }
}