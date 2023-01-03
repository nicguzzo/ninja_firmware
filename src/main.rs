#![no_main]
#![no_std]
//#![feature(generic_const_exprs)]
use defmt_rtt as _; 

// global logger
use panic_probe as _;
//use defmt::Format;


#[defmt::panic_handler]
fn panic() -> ! {
    cortex_m::asm::udf()
}
use frunk::{HCons, HNil};    
use stm32f1xx_hal::device::I2C2;
use stm32f1xx_hal::gpio::{ErasedPin,Input,PushPull, PullUp, Output, Alternate, OpenDrain, PB10, PB11};
use stm32f1xx_hal::{prelude::*, gpio::PinState};
use stm32f1xx_hal::usb::{UsbBus};
use stm32f1xx_hal::{pac, usb::Peripheral};
//use stm32f1xx_hal::flash::FlashWriter;
//use cortex_m::peripheral::NVIC;
use stm32f1xx_hal::{timer::{Event,CounterUs},i2c::{BlockingI2c, Mode, DutyCycle}};
use usb_device::class_prelude::*;
use usb_device::prelude::*;
//use usbd_human_interface_device::device::consumer::{MultipleConsumerReport, ConsumerControlInterface};
//use usbd_human_interface_device::device::mouse::{WheelMouseReport, WheelMouseInterface};
//use usbd_human_interface_device::page::Consumer;
//use usbd_human_interface_device::device::keyboard::KeyboardLedsReport;
use usbd_human_interface_device::device::mouse::WheelMouseInterface;
use usbd_human_interface_device::device::consumer::ConsumerControlInterface;
use usbd_human_interface_device::page::Keyboard as Kc;
use usbd_human_interface_device::device::keyboard::NKROBootKeyboardInterface;
use usbd_human_interface_device::prelude::*;

use eeprom24x::{Eeprom24x, SlaveAddr};

use defmt::{info};
//use cortex_m::asm::delay;
mod config_class;
mod secondary_side;
mod keyboard;

use crate::keyboard::keyboard::KeyboardTrait;
use keyboard::keyboard::{Ninja, KB_N_BYTES};
use config_class::RawConfInterface;
use secondary_side::SecondarySideI2C;
use keyboard::key::Key;

const REPORT_BUFF_MAX:usize=42;
const EEPROM_MARK:u8 = 0xAB;

type UsbDev<'a>  = UsbDevice<'a, UsbBus<Peripheral>>;
type UsbKb<'a> =UsbHidClass<UsbBus<Peripheral>, 
        HCons<RawConfInterface<'a, UsbBus<Peripheral>>, 
        HCons<ConsumerControlInterface<'a, UsbBus<Peripheral>>, 
        HCons<WheelMouseInterface<'a, UsbBus<Peripheral>>, 
        HCons<NKROBootKeyboardInterface<'a,UsbBus<Peripheral>>, HNil>>>>>;


type I2cT=BlockingI2c::<I2C2, (PB10<Alternate<OpenDrain>>, PB11<Alternate<OpenDrain>>)>;

type I2cProxy = shared_bus::I2cProxy<'static, shared_bus::AtomicCheckMutex<I2cT>>;
type EepromT=Eeprom24x<I2cProxy,eeprom24x::page_size::B32,eeprom24x::addr_size::TwoBytes>;


type Rows = [ErasedPin<Input<PullUp>>; Ninja::ROWS];
type Cols = [ErasedPin<Output<PushPull>>; Ninja::COLS];


const CONF_KEY_BYTES:usize=2; //bytes per key in conf report
const CONF_SIZE:usize=Ninja::COLS*Ninja::ROWS*Ninja::LAYERS*Ninja::SIDES*CONF_KEY_BYTES+2;//2 byte mark size
const PAGE_SIZE:usize=32;
const CONF_PAGES:usize=(CONF_SIZE>>5)+1;
const CONF_PAGES_SIZE:usize=PAGE_SIZE*CONF_PAGES;

type Matrix= [u8;KB_N_BYTES];
type Matrices=[[Matrix;2];Ninja::SIDES];

type Side = [[Key; Ninja::COLS]; Ninja::ROWS];
type Layers=[Side;Ninja::LAYERS];
type Keys= [Layers;Ninja::SIDES];

type ReportBuff=[Kc;REPORT_BUFF_MAX];


pub struct NinjaKb{
    rows:Rows,
    cols:Cols,
    matrices:Matrices,
    keys:Keys,
    layer:usize,
    led:ErasedPin<Output<PushPull>>,
    report_buff:ReportBuff,    
    delay_eeprom_cycles:u32,
}
pub enum State{
    Idle,
    SendKbInfo,
    ReceiveKeys(u8,u8),
    RequestKeys(u8,u8),
    ResetEeprom,
    SendReport
}

pub struct I2cDevices {
    pub secondary_side: SecondarySideI2C<I2cProxy>,
    pub eeprom:EepromT
}

#[rtic::app(device = stm32f1xx_hal::pac)]
mod app {
    use crate::{*};
    #[shared]
    struct Shared {        
        usb_dev: UsbDev<'static>,
        hid_kb: UsbKb<'static>,        
        report_buff:ReportBuff,
        state:Option<State>,
        conf_report:config_class::RawConfMsg
    }

    #[local]
    struct Local {
        timer:CounterUs<pac::TIM2>,
        ninja_kb:NinjaKb,
        i2c_devices:I2cDevices,
        count_10ms:u8,
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local, init::Monotonics){    
        static mut USB_BUS:Option<UsbBusAllocator<UsbBus<Peripheral>>>=None;
        
        let mut flash = cx.device.FLASH.constrain();
        let rcc = cx.device.RCC.constrain();
        let clocks = rcc
                .cfgr
                .use_hse(8.MHz())
                .sysclk(72.MHz())
                .pclk1(48.MHz())
                .freeze(&mut flash.acr);

        assert!(clocks.usbclk_valid());

        let mut afio  = cx.device.AFIO.constrain();
        let mut gpioa = cx.device.GPIOA.split();
        let mut gpiob = cx.device.GPIOB.split();
        let mut gpioc = cx.device.GPIOC.split();
        
        //disable jtag pins
        let (_gpioa_pa15, gpiob_pb3, gpiob_pb4) = afio.mapr.disable_jtag(gpioa.pa15, gpiob.pb3, gpiob.pb4);
        
        let mut led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh).erase();
        led.set_high();
      
        let layer:usize=0;
  
        //key pins, this can't be defeined elsewehere, 'cause Peripheral move reasons...

        #[cfg(feature="model_corne")]
        let rows:Rows=[
            gpiob.pb5.into_pull_up_input(&mut gpiob.crl).erase(),
            gpiob.pb6.into_pull_up_input(&mut gpiob.crl).erase(),
            gpiob.pb7.into_pull_up_input(&mut gpiob.crl).erase(),
            gpiob.pb8.into_pull_up_input(&mut gpiob.crh).erase()
        ];
        
        #[cfg(feature="model_corne")]
        let cols:Cols= [
            gpiob.pb12.into_push_pull_output_with_state(&mut gpiob.crh,PinState::High).erase(),
            gpiob.pb13.into_push_pull_output_with_state(&mut gpiob.crh,PinState::High).erase(),
            gpiob.pb14.into_push_pull_output_with_state(&mut gpiob.crh,PinState::High).erase(),
            gpiob.pb15.into_push_pull_output_with_state(&mut gpiob.crh,PinState::High).erase(),
             gpiob_pb3.into_push_pull_output_with_state(&mut gpiob.crl,PinState::High).erase(),
             gpiob_pb4.into_push_pull_output_with_state(&mut gpiob.crl,PinState::High).erase()
        ];   
        
        #[cfg(feature="model_ninja1")]
        let rows:Rows=[
            gpiob.pb5.into_pull_up_input(&mut gpiob.crl).erase(),
            gpiob.pb6.into_pull_up_input(&mut gpiob.crl).erase(),
            gpiob.pb7.into_pull_up_input(&mut gpiob.crl).erase(),
            gpiob.pb8.into_pull_up_input(&mut gpiob.crh).erase(),
            gpiob.pb9.into_pull_up_input(&mut gpiob.crh).erase()
        ];
        
        #[cfg(feature="model_ninja1")]
        let cols:Cols= [
            gpiob.pb12.into_push_pull_output_with_state(&mut gpiob.crh,PinState::High).erase(),
            gpiob.pb13.into_push_pull_output_with_state(&mut gpiob.crh,PinState::High).erase(),
            gpiob.pb14.into_push_pull_output_with_state(&mut gpiob.crh,PinState::High).erase(),
            gpiob.pb15.into_push_pull_output_with_state(&mut gpiob.crh,PinState::High).erase(),
             gpiob_pb3.into_push_pull_output_with_state(&mut gpiob.crl,PinState::High).erase(),
             gpiob_pb4.into_push_pull_output_with_state(&mut gpiob.crl,PinState::High).erase()
        ]; 


        //keyboard matrix
        let matrices:Matrices=[[[0u8;KB_N_BYTES];2];Ninja::SIDES];
        
        let keys:Keys=Ninja::get_default_keys();

        let report_buff:ReportBuff = [Kc::NoEventIndicated;REPORT_BUFF_MAX];
        
        info!("size {}",CONF_SIZE);
        
        //i2c
        //let i2c=None;
        led.set_low();                
        for _i in 0..30 {
            cortex_m::asm::delay(clocks.sysclk().raw() / 100);
            led.toggle();
        }
        led.set_high();                
        info!("Conf i2c.");
        let sda = gpiob.pb11.into_alternate_open_drain(&mut gpiob.crh);
        let scl = gpiob.pb10.into_alternate_open_drain(&mut gpiob.crh);
            
        let i2c = BlockingI2c::i2c2(
            cx.device.I2C2,
            (scl, sda),
            Mode::Fast {
                frequency: 400.kHz(),
                duty_cycle: DutyCycle::Ratio16to9,
            },
            clocks,
            1000,
            10,
            1000,
            1000,
        );

        let i2c_bus: &'static _ =shared_bus::new_atomic_check!(I2cT = i2c).unwrap();
        let address = SlaveAddr::default();
        let eeprom = Eeprom24x::new_24x32(i2c_bus.acquire_i2c(), address);
        let secondary_side=SecondarySideI2C::new(i2c_bus.acquire_i2c());
        let i2c_devices=I2cDevices{secondary_side,eeprom};
        
        info!("Conf i2c done.");

        info!("Conf tick timer.");
        let mut timer = cx.device.TIM2.counter_us(&clocks);
        match timer.start(1.millis()){
            Ok(_)=>timer.listen(Event::Update),
            Err(_)=> info!("tick timer error.")
        }
        info!("Conf tick timer done.");

        //USB
        info!("Start Usb");
        let mut usb_dp = gpioa.pa12.into_push_pull_output(&mut gpioa.crh);
        usb_dp.set_low();
        //for _i in 0..100 {
            cortex_m::asm::delay(clocks.sysclk().raw() / 100);
        //}

        let usb = Peripheral {
            usb: cx.device.USB,
            pin_dm: gpioa.pa11,
            pin_dp: usb_dp.into_floating_input(&mut gpioa.crh),
        };        
        let usb_bus = UsbBus::new(usb);

        unsafe {
            USB_BUS.replace(usb_bus);
        }

        let nkro=usbd_human_interface_device::device::keyboard::NKROBootKeyboardInterface::default_config();
        let mouse=usbd_human_interface_device::device::mouse::WheelMouseInterface::default_config();
        let consumer=usbd_human_interface_device::device::consumer::ConsumerControlInterface::default_config();

        let config=RawConfInterface::default_config();
        
        //control.inner_config.description=Some("Ninja Keyboard Corne Control");
        
        let usb_bus= match unsafe { USB_BUS.as_ref() } {
            Some(usb_bus)=> {
                usb_bus
            },
            _=>{                
                panic!("no usb_bus")
            }
        };

        let hid_kb:UsbKb  = UsbHidClassBuilder::new()
                    .add_interface(nkro)
                    .add_interface(mouse)
                    .add_interface(consumer)
                    .add_interface(config)
                    .build(usb_bus);
                
        let usb_dev:UsbDev = UsbDeviceBuilder::new(usb_bus, UsbVidPid(0xcaca, 0x0001))
        .manufacturer("Nicguzzo")
        .product("Ninja Keyboard Corne")
        .serial_number("0")
        .build();

        info!("Usb done.");
        led.set_high();
        
        let ninja_kb= NinjaKb{
            rows,
            cols,
            matrices,
            keys,
            report_buff,
            layer,
            led,            
            delay_eeprom_cycles:clocks.sysclk().raw()/50
        };
        let state=None;
        let count_10ms=0;
        
        let conf_report=config_class::RawConfMsg{packet:[0u8;64]};
        
        (Shared {  usb_dev, hid_kb,report_buff,state,conf_report }, Local {timer,ninja_kb,i2c_devices,count_10ms}, init::Monotonics())
    }

    #[task(binds = USB_HP_CAN_TX, priority = 2, shared = [usb_dev, hid_kb])]
    fn usb_tx(cx: usb_tx::Context) {
        let mut usb_dev = cx.shared.usb_dev;
        let mut hid_kb = cx.shared.hid_kb;
        (&mut usb_dev, &mut hid_kb).lock(|usb_dev, hid_kb| {
            usb_poll(usb_dev, hid_kb);
        });
    }
    #[task(binds = USB_LP_CAN_RX0, priority = 2, shared = [usb_dev, hid_kb])]
    fn usb_rx(cx: usb_rx::Context) {
        let mut usb_dev = cx.shared.usb_dev;
        let mut hid_kb = cx.shared.hid_kb;
        (&mut usb_dev, &mut hid_kb).lock(|usb_dev, hid_kb| {
            usb_poll(usb_dev, hid_kb);
        });
    }
    #[task(binds = TIM2, priority = 3, shared = [hid_kb,report_buff,state,conf_report], local=[timer,count_10ms])]
    fn tick(cx: tick::Context) {
        let mut hid_kb = cx.shared.hid_kb;
        let mut report_buff = cx.shared.report_buff;
        let mut state=cx.shared.state;
        let mut conf_report=cx.shared.conf_report;
        
        
        (&mut hid_kb,&mut report_buff,&mut state,&mut conf_report).lock(|hid_kb,report_buff,state,conf_report| {
            let keyboard = hid_kb.interface::<NKROBootKeyboardInterface<'_, _>, _>();

            if *cx.local.count_10ms > 10 {
                *cx.local.count_10ms=0;
            }else{
                *cx.local.count_10ms+=1;
            }
            match keyboard.write_report(*report_buff) {
                Err(UsbHidError::WouldBlock) => {info!("WouldBlock")}
                //Err(UsbHidError::Duplicate) => {info!("Duplicate")}
                Ok(_) => {}
                Err(_e) => {
                    //info!("Failed to write keyboard report: ")
                }
            };
            match keyboard.read_report(){
                Err(UsbError::WouldBlock) => {},                    
                Ok(_leds) => { 
                    info!("read led report")
                }
                Err(_e) => {
                    info!("Failed to read keyboard report: ")
                }
            }
            let control = hid_kb.interface::<RawConfInterface<'_, _>, _>();
            match control.read_report(){
                Err(UsbError::WouldBlock) => {},                    
                Ok(s) => { 
                    //info!("read conf report {}",s.packet);
                    match s.packet[0]{
                        0=>{//conf app requests kb info
                            info!("read_report SendKbInfo");
                            *state=Some(State::SendKbInfo);
                        },
                        1=>{//conf app sends keys conf
                            info!("read_report ReceiveKeys");
                            conf_report.packet=s.packet;
                            *state=Some(State::ReceiveKeys(s.packet[1],s.packet[2]));
                        },
                        2=>{//conf app requests keys conf
                            info!("read_report RequestKeys");
                            *state=Some(State::RequestKeys(s.packet[1],s.packet[2]));
                        },
                        3=>{
                            *state=Some(State::ResetEeprom);
                        },
                        _=>()
                    }
                }
                Err(_e) => {
                    info!("Failed to read conf report: ")
                }
            }
            if *cx.local.count_10ms == 10 {
                match state{
                    Some(State::SendReport)=>{
                        match control.write_report(conf_report){                
                            Ok(_) => { 
                                info!("control report write ok")
                            }
                            Err(_e) => {
                                info!("Failed to write control report")
                            }
                        }
                        *state=Some(State::Idle);
                    },
                    _=>()
                }
            }
            
        });
        cx.local.timer.clear_interrupt(Event::Update);
    }
    #[idle( local=[i2c_devices,ninja_kb],shared=[report_buff,state,conf_report])]
    fn idle(cx: idle::Context) -> ! {
        info!("idle");
        let i2c_devices=cx.local.i2c_devices;
        let ninja_kb=cx.local.ninja_kb;
        let mut state=cx.shared.state;
        let mut report_buff=cx.shared.report_buff;
        let mut conf_report=cx.shared.conf_report;

        cortex_m::asm::delay(ninja_kb.delay_eeprom_cycles);
        info!("read secondary_side keys");
        match i2c_devices.secondary_side.read_keys(){
            Ok(_)=>{
                info!("right side found");
            },
            Err(_)=>info!("eeprom read_byte error")
        }

        cortex_m::asm::delay(ninja_kb.delay_eeprom_cycles);
        info!("read conf from eeprom");
        let memory_address = 0;
        match i2c_devices.eeprom.read_byte(memory_address){
            Ok(_)=>{
                info!("eeprom read_byte");
                let read_data = i2c_devices.eeprom.read_byte(memory_address).unwrap();        
                info!("mark {}", read_data);
                if read_data!=EEPROM_MARK{
                    write_conf_to_eeprom(&mut ninja_kb.keys,&mut i2c_devices.eeprom,ninja_kb.delay_eeprom_cycles);
                }else{
                    info!("reading keys");
                    read_conf_from_eeprom(&mut ninja_kb.keys,&mut i2c_devices.eeprom);
                    info!("keys {}",ninja_kb.keys);
                }
            },
            Err(_)=>info!("eeprom read_byte error")
        }

        loop {
            //cortex_m::asm::nop();
            if update_kb_state(ninja_kb,&mut i2c_devices.secondary_side){
                (&mut report_buff).lock(|report_buff| {
                    for i in 0..REPORT_BUFF_MAX {
                        report_buff[i]=ninja_kb.report_buff[i];
                    }
                });
            }
            (&mut state,&mut conf_report).lock(|state,conf_report| {
                if let Some(state)=state{
                    match state{
                        State::SendKbInfo=>{
                            info!("idle SendKbInfo");
                            conf_report.packet[0]=0;//kbinfo
                            conf_report.packet[1]=Ninja::SIDES as u8;
                            conf_report.packet[2]=Ninja::LAYERS as u8;
                            conf_report.packet[3]=Ninja::ROWS as u8;
                            conf_report.packet[4]=Ninja::COLS as u8;
                            *state=State::SendReport;
                        },                        
                        State::ReceiveKeys(side,layer)=>{
                            info!("idle ReceiveKeys");
                            deserialize_keys(&conf_report.packet,&mut ninja_kb.keys);
                            *state=State::Idle;

                        },
                        State::RequestKeys(side,layer)=>{
                            info!("idle RequestKeys");
                            if (*side as usize) < Ninja::SIDES && (*layer as usize ) < Ninja::LAYERS{
                                serialize_keys(*side,*layer,&ninja_kb.keys[*side as usize][*layer as usize],&mut conf_report.packet);
                                *state=State::SendReport;
                            }else{
                                *state=State::Idle;
                            }
                        },
                        State::ResetEeprom=>{
                            info!("idle ResetEeprom");
                            i2c_devices.eeprom.write_byte(memory_address,0).unwrap();
                            *state=State::Idle;
                        },
                        State::SendReport=>{
                            info!("idle SendReport");
                            //*state=State::Idle;
                        }
                        State::Idle=>{
                            cortex_m::asm::nop();
                            //cortex_m::asm::wfi();
                        }
                    }
                }
            })
        }
    }    
}

fn usb_poll(usb_dev: &mut UsbDev, keyboard: &mut UsbKb) {
    if usb_dev.poll(&mut [keyboard]) {
        keyboard.poll();
    }
}

fn update_kb_state(ninja_kb:&mut NinjaKb ,secondary_side:&mut SecondarySideI2C<I2cProxy>)-> bool{
    let mut event=false;
    for byte in 0..KB_N_BYTES  {
        ninja_kb.matrices[0][1][byte]=ninja_kb.matrices[0][0][byte];
        ninja_kb.matrices[1][1][byte]=ninja_kb.matrices[1][0][byte];
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
    for side in 0..Ninja::SIDES{
        for col in 0..Ninja::COLS  {
            for row in 0..Ninja::ROWS{
                let index=row*Ninja::COLS+col;
                let byte=index>>3;
                let bit=(index%8) as u8;
                //pressed        
                let m1=ninja_kb.matrices[side][Ninja::MAIN][byte] & (1<<bit);
                let m2=ninja_kb.matrices[side][Ninja::SECONDARY][byte] & (1<<bit);
                if m1!=0 && m2==0{
                    ninja_kb.led.set_low();
                    match ninja_kb.keys[side][ninja_kb.layer][row][col]{
                        Key::Layer=>{
                            ninja_kb.layer=1;
                            event=false;
                            continue;
                        },
                        Key::Code(code)=>{
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
                        Key::Layer=>{
                            ninja_kb.layer=0;
                            event=false;
                            continue;
                        },
                        Key::Code(code)=>{
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

fn serialize_key(key:&Key)->(u8,u8){
    match key{
        Key::Code(code)=>(0,*code as u8),
        Key::Layer=>(1,0),
        Key::NoKey=>(2,0)
    }
}
fn deserialize_key(b1:u8,b2:u8)->Key{
    match b1{
        0=> Key::Code(Kc::from(b2)),
        1=> Key::Layer,
        2=>Key::NoKey,
        _=>Key::NoKey
    }
}
fn serialize_keys(side:u8,layer:u8,side_data:&Side,bytes:&mut [u8;64]){
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
fn deserialize_keys(bytes:&[u8;64],keys:&mut Keys){
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
fn write_conf_to_eeprom(keys:&mut Keys,eeprom:&mut EepromT,delay:u32){
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

fn read_conf_from_eeprom(keys:&mut Keys,eeprom:&mut EepromT){
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
/*fn reset(keys:&mut Keys,keys_f:&Keys){
    for side in 0..SIDES{
        for layer in 0..LAYERS{
            for col in 0..COLS{
                for row in 0..ROWS{
                    keys[side][layer][row][col]=keys_f[side][layer][row][col];
                }
            }
        }
    }
}*/
/*fn show(keys:&Keys){
    for side in 0..SIDES{
        info!("Side {}",side);
        for layer in 0..LAYERS{
            info!("Side {}",layer);
            for row in 0..ROWS{
                info!("{}",keys[side][layer][row]);
            }
        }
    }
}*/