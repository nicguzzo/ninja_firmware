#![no_main]
#![no_std]

use defmt_rtt as _; 
// global logger
use panic_probe as _;
use defmt::Format;


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
use usbd_human_interface_device::device::consumer::{MultipleConsumerReport, ConsumerControlInterface};
use usbd_human_interface_device::device::mouse::{WheelMouseReport, WheelMouseInterface};
use usbd_human_interface_device::page::{Keyboard,Consumer};
use usbd_human_interface_device::device::keyboard::{KeyboardLedsReport, NKROBootKeyboardInterface};
use usbd_human_interface_device::prelude::*;

use eeprom24x::{Eeprom24x, SlaveAddr};

use defmt::{info};
//use cortex_m::asm::delay;
mod config_class;
mod right_side;
use config_class::RawConfInterface;
use right_side::RightSideI2C;

const REPORT_BUFF_MAX:usize=42;
const COLS:usize=6;
const ROWS:usize=4;
const LAYERS:usize=4;
const SIDES:usize=2;
const CONF_KEY_BYTES:usize=2; //bytes per key in conf report
const EEPROM_MARK:u8 = 0xAB;
const CONF_SIZE:usize=COLS*ROWS*LAYERS*SIDES*CONF_KEY_BYTES+2;//2 byte mark size
const PAGE_SIZE:usize=32;
const CONF_PAGES:usize=(CONF_SIZE>>5)+1;
const CONF_PAGES_SIZE:usize=PAGE_SIZE*CONF_PAGES;

type UsbDev<'a>  = UsbDevice<'a, UsbBus<Peripheral>>;

type UsbKb<'a> =UsbHidClass<UsbBus<Peripheral>, 
        HCons<RawConfInterface<'a, UsbBus<Peripheral>>, 
        HCons<ConsumerControlInterface<'a, UsbBus<Peripheral>>, 
        HCons<WheelMouseInterface<'a, UsbBus<Peripheral>>, 
        HCons<NKROBootKeyboardInterface<'a,UsbBus<Peripheral>>, HNil>>>>>;


type I2cT=BlockingI2c::<I2C2, (PB10<Alternate<OpenDrain>>, PB11<Alternate<OpenDrain>>)>;

type I2cProxy = shared_bus::I2cProxy<'static, shared_bus::AtomicCheckMutex<I2cT>>;
type EepromT=Eeprom24x<I2cProxy,eeprom24x::page_size::B32,eeprom24x::addr_size::TwoBytes>;
type Matrix= [[bool; COLS]; ROWS];

type Rows = [ErasedPin<Input<PullUp>>; ROWS];
type Cols = [ErasedPin<Output<PushPull>>; COLS];

type ReportBuff=[Keyboard;REPORT_BUFF_MAX];

#[derive(Clone,Copy)]
enum Key{
    Code(Keyboard),
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
type Side = [[Key; COLS]; ROWS];
type Layers=[Side;LAYERS];
type Keys= [Layers;SIDES];
/*struct Keys{
    left:Layers,
    right:Layers
}*/
pub struct NinjaKb{
    rows:Rows,
    cols:Cols,
    matrix:Matrix,    
    matrix_last:Matrix,
    sec_matrix:Matrix,
    sec_matrix_last:Matrix,
    keys:Keys,
    //factorykeys:Keys,
    report_buff:ReportBuff,
    layer:usize,
    led:ErasedPin<Output<PushPull>>,
    send_side_idx:u8,
    send_layer_idx:u8,
    count_10ms:u8,
    send_kb_info:bool,
    read_from_eeprom_at_start:bool,
    eeprom_in_use:bool,
    reset_eeprom:bool,
    delay_eeprom_cycles:u32
}

pub struct I2cDevices {
    pub right_side: RightSideI2C<I2cProxy>,
    pub eeprom:EepromT
    
}

#[rtic::app(device = stm32f1xx_hal::pac)]
mod app {
    

    use crate::{*};
    #[shared]
    struct Shared {        
        usb_dev: UsbDev<'static>,
        hid_kb: UsbKb<'static>,
        ninja_kb:NinjaKb,
        i2c_devices:I2cDevices
    }

    #[local]
    struct Local {
        timer:CounterUs<pac::TIM2>,
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local, init::Monotonics){    
        static mut USB_BUS:Option<UsbBusAllocator<UsbBus<Peripheral>>>=None;
        //static mut I2C_BUS:Option<I2cProxy<'static, NullMutex<I2cT>>>=None;
        
        let mut flash = cx.device.FLASH.constrain();
        let rcc = cx.device.RCC.constrain();
        let clocks = rcc
                .cfgr
                .use_hse(8.MHz())
                .sysclk(72.MHz())
                .pclk1(48.MHz())
                .freeze(&mut flash.acr);

        /*let clocks = rcc
                .cfgr
                .use_hse(8.MHz())
                .sysclk(72.MHz())
                .pclk1(36.MHz())
                .freeze(&mut flash.acr);*/

        /*let clocks = rcc
                .cfgr
                .use_hse(8.MHz())
                .sysclk(48.MHz())
                .pclk1(24.MHz())
                .freeze(&mut flash.acr);*/

        assert!(clocks.usbclk_valid());

        //let mut delay = cx.core.SYST.delay(&clocks);

        let mut afio = cx.device.AFIO.constrain();
        let mut gpioa = cx.device.GPIOA.split();
        let mut gpiob = cx.device.GPIOB.split();
        let mut gpioc = cx.device.GPIOC.split();
        
        //disable jtag pins
        let (_gpioa_pa15, gpiob_pb3, gpiob_pb4) = afio.mapr.disable_jtag(gpioa.pa15, gpiob.pb3, gpiob.pb4);
        //let mut timer = Timer::syst(cp.SYST, &clocks).counter_us();

        let mut led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh).erase();
        led.set_high();
  
        let row0 =gpiob.pb5.into_pull_up_input(&mut gpiob.crl).erase();
        let row1 =gpiob.pb6.into_pull_up_input(&mut gpiob.crl).erase();
        let row2 =gpiob.pb7.into_pull_up_input(&mut gpiob.crl).erase();
        let row3 =gpiob.pb8.into_pull_up_input(&mut gpiob.crh).erase();

        let col0 = gpiob.pb12.into_push_pull_output_with_state(&mut gpiob.crh,PinState::High).erase();
        let col1 = gpiob.pb13.into_push_pull_output_with_state(&mut gpiob.crh,PinState::High).erase();
        let col2 = gpiob.pb14.into_push_pull_output_with_state(&mut gpiob.crh,PinState::High).erase();
        let col3 = gpiob.pb15.into_push_pull_output_with_state(&mut gpiob.crh,PinState::High).erase();
        let col4 =  gpiob_pb3.into_push_pull_output_with_state(&mut gpiob.crl,PinState::High).erase();
        let col5 =  gpiob_pb4.into_push_pull_output_with_state(&mut gpiob.crl,PinState::High).erase();

        
        let layer:usize=0;
        let rows:Rows=[row0,row1,row2,row3];
        let cols:Cols=[col0,col1,col2,col3,col4,col5];

        //keyboard matrix
        
        let matrix:Matrix=[ [false; COLS]; ROWS];    
        let matrix_last:Matrix= [ [false; COLS]; ROWS];
    
        let sec_matrix:Matrix=[ [false; COLS]; ROWS];    
        let sec_matrix_last:Matrix= [ [false; COLS]; ROWS];
    
        //let mut matrix_debounce:[[u64; COLS]; ROWS]= [ [0; COLS]; ROWS];
    
        
        let keys:Keys=[
            [
                [
                    [Key::Code(Keyboard::Escape),Key::Code(Keyboard::Q),Key::Code(Keyboard::W),Key::Code(Keyboard::E),Key::Code(Keyboard::R),Key::Code(Keyboard::T)],
                    [Key::Code(Keyboard::Tab),Key::Code(Keyboard::A),Key::Code(Keyboard::S),Key::Code(Keyboard::D),Key::Code(Keyboard::F),Key::Code(Keyboard::G)],
                    [Key::Code(Keyboard::LeftShift),Key::Code(Keyboard::Z),Key::Code(Keyboard::X),Key::Code(Keyboard::C),Key::Code(Keyboard::V),Key::Code(Keyboard::B)],
                    [Key::NoKey,Key::NoKey,Key::NoKey,Key::Code(Keyboard::LeftControl),Key::Code(Keyboard::LeftGUI),Key::Layer ],
                ],
                [
                    [Key::Code(Keyboard::F1),Key::Code(Keyboard::F2),Key::Code(Keyboard::F3),Key::Code(Keyboard::F4),Key::Code(Keyboard::F5),Key::Code(Keyboard::F6) ],
                    [Key::Code(Keyboard::Keyboard1),Key::Code(Keyboard::Keyboard1),Key::Code(Keyboard::Keyboard2),Key::Code(Keyboard::Keyboard3),Key::Code(Keyboard::Keyboard4),Key::Code(Keyboard::Keyboard5)],
                    [Key::Code(Keyboard::Backslash),Key::Code(Keyboard::Z),Key::Code(Keyboard::X) ,Key::Code(Keyboard::C), Key::Code(Keyboard::V) ,Key::Code(Keyboard::B)  ],
                    [Key::NoKey,Key::NoKey,Key::NoKey ,Key::Code(Keyboard::LeftAlt),Key::Code(Keyboard::RightGUI),Key::Layer ],
                ],
                [
                    [Key::NoKey,Key::NoKey,Key::NoKey,Key::NoKey,Key::NoKey,Key::NoKey],
                    [Key::NoKey,Key::NoKey,Key::NoKey,Key::NoKey,Key::NoKey,Key::NoKey],
                    [Key::NoKey,Key::NoKey,Key::NoKey,Key::NoKey,Key::NoKey,Key::NoKey],
                    [Key::NoKey,Key::NoKey,Key::NoKey,Key::NoKey,Key::NoKey,Key::NoKey],
                ]
                ,
                [
                    [Key::NoKey,Key::NoKey,Key::NoKey,Key::NoKey,Key::NoKey,Key::NoKey],
                    [Key::NoKey,Key::NoKey,Key::NoKey,Key::NoKey,Key::NoKey,Key::NoKey],
                    [Key::NoKey,Key::NoKey,Key::NoKey,Key::NoKey,Key::NoKey,Key::NoKey],
                    [Key::NoKey,Key::NoKey,Key::NoKey,Key::NoKey,Key::NoKey,Key::NoKey],
                ]
            ],
            [
                [
                    [Key::Code(Keyboard::Y), Key::Code(Keyboard::U),Key::Code(Keyboard::I),Key::Code(Keyboard::O),Key::Code(Keyboard::P),Key::Code(Keyboard::DeleteBackspace) ],
                    [Key::Code(Keyboard::H), Key::Code(Keyboard::J),Key::Code(Keyboard::K),Key::Code(Keyboard::L),Key::Code(Keyboard::Semicolon),Key::Code(Keyboard::Backslash) ],
                    [Key::Code(Keyboard::N), Key::Code(Keyboard::M),Key::Code(Keyboard::Comma),Key::Code(Keyboard::LeftBrace),Key::Code(Keyboard::RightBrace),Key::Code(Keyboard::Apostrophe) ],
                    [Key::Code(Keyboard::ReturnEnter),Key::Code(Keyboard::Space),Key::Code(Keyboard::Dot) ,Key::NoKey,Key::NoKey,Key::NoKey]
                ],
                [
                    [Key::Code(Keyboard::F7)    ,Key::Code(Keyboard::F8)  ,Key::Code(Keyboard::F9)    ,Key::Code(Keyboard::F10)   ,Key::Code(Keyboard::F11)     ,Key::Code(Keyboard::F12) ],
                    [Key::Code(Keyboard::Keyboard6),Key::Code(Keyboard::UpArrow)  ,Key::Code(Keyboard::Keyboard7),Key::Code(Keyboard::Keyboard8),Key::Code(Keyboard::Keyboard9),Key::Code(Keyboard::Keyboard0)],
                    [Key::Code(Keyboard::LeftArrow)  ,Key::Code(Keyboard::DownArrow),Key::Code(Keyboard::RightArrow) ,Key::Code(Keyboard::PageUp),Key::Code(Keyboard::PageDown),Key::Code(Keyboard::Minus) ],
                    [Key::Code(Keyboard::DeleteForward),Key::Code(Keyboard::Home),Key::Code(Keyboard::End)   ,Key::NoKey           ,Key::NoKey              ,Key::NoKey]
                ],
                [
                    [Key::NoKey,Key::NoKey,Key::NoKey,Key::NoKey,Key::NoKey,Key::NoKey],
                    [Key::NoKey,Key::NoKey,Key::NoKey,Key::NoKey,Key::NoKey,Key::NoKey],
                    [Key::NoKey,Key::NoKey,Key::NoKey,Key::NoKey,Key::NoKey,Key::NoKey],
                    [Key::NoKey,Key::NoKey,Key::NoKey,Key::NoKey,Key::NoKey,Key::NoKey],
                ]
                ,
                [
                    [Key::NoKey,Key::NoKey,Key::NoKey,Key::NoKey,Key::NoKey,Key::NoKey],
                    [Key::NoKey,Key::NoKey,Key::NoKey,Key::NoKey,Key::NoKey,Key::NoKey],
                    [Key::NoKey,Key::NoKey,Key::NoKey,Key::NoKey,Key::NoKey,Key::NoKey],
                    [Key::NoKey,Key::NoKey,Key::NoKey,Key::NoKey,Key::NoKey,Key::NoKey],
                ]
            ]
        ];

        //let mut keys:Keys=[[[[Key::NoKey; COLS];ROWS];LAYERS];SIDES];
        
        //let keys=factorykeys;
        
        let report_buff:ReportBuff = [Keyboard::NoEventIndicated;REPORT_BUFF_MAX];
        
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
        let right_side=RightSideI2C::new(i2c_bus.acquire_i2c());
        let i2c_devices=I2cDevices{right_side,eeprom};
        
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
            matrix,
            matrix_last,
            sec_matrix,
            sec_matrix_last,
            keys,
            report_buff,
            layer,
            led,
            send_side_idx:SIDES as u8,
            send_layer_idx:0,
            count_10ms:0,
            send_kb_info:false,
            read_from_eeprom_at_start:true,
            eeprom_in_use:false,
            reset_eeprom:false,
            delay_eeprom_cycles:clocks.sysclk().raw()/50
        };
                
        
        (Shared {  usb_dev, hid_kb,i2c_devices, ninja_kb }, Local {timer }, init::Monotonics())
    }

    #[task(binds = USB_HP_CAN_TX, priority = 2, shared = [usb_dev, hid_kb, ninja_kb])]
    fn usb_tx(cx: usb_tx::Context) {
        let mut usb_dev = cx.shared.usb_dev;
        let mut hid_kb = cx.shared.hid_kb;
        let mut ninja_kb = cx.shared.ninja_kb;
        (&mut usb_dev, &mut hid_kb,&mut ninja_kb).lock(|usb_dev, hid_kb,_ninja_kb| {
            usb_poll(usb_dev, hid_kb);
        });
    }

    #[task(binds = USB_LP_CAN_RX0, priority = 2, shared = [usb_dev, hid_kb, ninja_kb])]
    fn usb_rx(cx: usb_rx::Context) {
        let mut usb_dev = cx.shared.usb_dev;
        let mut hid_kb = cx.shared.hid_kb;
        let mut ninja_kb = cx.shared.ninja_kb;
        (&mut usb_dev, &mut hid_kb,&mut ninja_kb).lock(|usb_dev, hid_kb,_ninja_kb| {
            usb_poll(usb_dev, hid_kb);
        });
    }
    #[task(binds = TIM2, priority = 3, shared = [hid_kb,i2c_devices, ninja_kb], local=[timer])]
    fn tick(cx: tick::Context) {
        let mut hid_kb = cx.shared.hid_kb;
        let mut ninja_kb = cx.shared.ninja_kb;
        let mut i2c_devices=cx.shared.i2c_devices;
        //info!("tick");
        //cx.local.ninja_kb.led.set_low();
        (&mut hid_kb,&mut ninja_kb,&mut i2c_devices).lock(|hid_kb,ninja_kb,i2c_devices| {
            let keyboard = hid_kb.interface::<NKROBootKeyboardInterface<'_, _>, _>();
            //let control = hid_kb.interface::<ConsumerControlInterface<'_, _>, _>();
            /*if ninja_kb.read_from_eeprom_at_start {
                ninja_kb.read_from_eeprom_at_start=false;
                ninja_kb.eeprom_in_use=true;
                info!("read conf from eeprom");
                //cortex_m::interrupt::disable();
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
                            //read_conf_from_eeprom(&mut ninja_kb.keys,&mut i2c_devices.eeprom);
                            info!("keys");
                            //show(&ninja_kb.keys);
                        }
                    },
                    Err(_)=>info!("eeprom read_byte error")
                }
                ninja_kb.eeprom_in_use=false;
                //unsafe{cortex_m::interrupt::enable();}
            }*/
            if ninja_kb.reset_eeprom{
                ninja_kb.reset_eeprom=false;
                info!("reset keys");
                //info!("factorykeys {}",ninja_kb.factorykeys);
                //show(&ninja_kb.factorykeys);
                //info!("keys {}",ninja_kb.keys);
                //show(&ninja_kb.keys);
                //reset(&mut ninja_kb.keys,&ninja_kb.factorykeys);
                //info!("reset conf to eeprom");
                //write_conf_to_eeprom(&mut ninja_kb.factorykeys,&mut i2c_devices.eeprom,ninja_kb.delay_eeprom_cycles);
            }

            if ninja_kb.count_10ms > 10 {
                ninja_kb.count_10ms=0;
            }else{
                ninja_kb.count_10ms+=1;
            }
            if ninja(ninja_kb,&mut i2c_devices.right_side)
            {
                match keyboard.write_report(ninja_kb.report_buff) {
                    Err(UsbHidError::WouldBlock) => {info!("WouldBlock")}
                    Err(UsbHidError::Duplicate) => {info!("Duplicate")}
                    Ok(_) => {}
                    Err(_e) => {
                        info!("Failed to write keyboard report: ")
                    }
                };
            }
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
            //let data = &mut [0;64];
            match control.read_report(){
                Err(UsbError::WouldBlock) => {},                    
                Ok(s) => { 
                    info!("read conf report {}",s.packet);
                    match s.packet[0]{
                        0=>{//conf app requests kb info
                            ninja_kb.send_kb_info=true;
                            info!("send_kb_info");
                        },
                        1=>{//conf app sends keys conf
                            deserialize_keys(&s.packet,&mut ninja_kb.keys);
                            info!("deserialize_keys");
                        },
                        2=>{//conf app requests keys conf
                            ninja_kb.send_side_idx=0;
                            ninja_kb.send_layer_idx=0;
                        },
                        3=>{
                            ninja_kb.reset_eeprom=true;
                        },
                        _=>()
                    }
                }
                Err(_e) => {
                    info!("Failed to read conf report: ")
                }
            }
            if ninja_kb.count_10ms == 10 {                
                if ninja_kb.send_kb_info{
                    ninja_kb.send_kb_info=false;
                    let mut packet=[0u8;64];
                    packet[0]=0;//kbinfo
                    packet[1]=SIDES as u8;
                    packet[2]=LAYERS as u8;
                    packet[3]=ROWS as u8;
                    packet[4]=COLS as u8;
                    let report=config_class::RawConfMsg{packet};
                    match control.write_report(&report){                
                        Ok(_) => { 
                            info!("report write ok")
                        }
                        Err(_e) => {
                            info!("Failed to write_report")
                        }
                    }
                }else{
                    let  side=ninja_kb.send_side_idx as usize;
                    let layer=ninja_kb.send_layer_idx as usize;
                    if side < SIDES {
                        if layer< LAYERS{
                            info!("side: {}",side);
                            info!("layer: {}",layer);
                            let packet=serialize_keys(ninja_kb.send_side_idx,ninja_kb.send_layer_idx,&ninja_kb.keys[side][layer]);
                            let report=config_class::RawConfMsg{packet};
                            match control.write_report(&report){                
                                Ok(_) => { 
                                    info!("report write ok")
                                }
                                Err(_e) => {
                                    info!("Failed to write_report")
                                }
                            }
                            ninja_kb.send_layer_idx+=1;
                        }else{
                            ninja_kb.send_layer_idx=0;
                            ninja_kb.send_side_idx+=1;
                        }
                    }
                }
            }
            
        });
        cx.local.timer.clear_interrupt(Event::Update);
    }
    #[idle(/*shared=[i2c_devices]*/)]
    fn idle(cx: idle::Context) -> ! {
        /*info!("idle");
        let i2c_devices=cx.shared.i2c_devices;
        
        info!("read conf from eeprom");
        //cortex_m::interrupt::disable();
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
                    //read_conf_from_eeprom(&mut ninja_kb.keys,&mut i2c_devices.eeprom);
                    info!("keys");
                    //show(&ninja_kb.keys);
                }
            },
            Err(_)=>info!("eeprom read_byte error")
        }*/
        loop {
            //cortex_m::asm::nop();
            cortex_m::asm::wfi();
        }
    }    
}

fn usb_poll(usb_dev: &mut UsbDev, keyboard: &mut UsbKb) {
    if usb_dev.poll(&mut [keyboard]) {
        keyboard.poll();
    }
}

fn ninja(ninja_kb:&mut NinjaKb ,right_side:&mut RightSideI2C<I2cProxy>)-> bool{
    let mut event=false;
    for col in 0..COLS  {
        ninja_kb.cols[col].set_low();
        for row in 0..ROWS {
            ninja_kb.matrix_last[row][col]=ninja_kb.matrix[row][col];
            ninja_kb.matrix[row][col]=ninja_kb.rows[row].is_low();
        }
        ninja_kb.cols[col].set_high();
    }
    if !ninja_kb.eeprom_in_use{
        match right_side.read_keys(){
            Ok(buffer) =>{
                //ninja_kb.led.set_low();
                let mut b;
                let mut k:usize=0;
                let mut bit:u8=7;
                for col in 0..COLS  {
                    for row in 0..ROWS{
                        ninja_kb.sec_matrix_last[row][col]=ninja_kb.sec_matrix[row][col];
                        b=k/8;
                        ninja_kb.sec_matrix[row][col]= ((buffer[b]>>bit)&0x01) != 0;
                        if bit==0{
                            bit=7;
                        }else{
                            bit=bit-1;
                        }
                        k+=1;
                    }
                }
            },
            Err(_) =>{
                info!("i2c read/write error")
            }
        }
    }
    //info!("matrix_l {}", ninja_kb.matrix);
    //info!("matrix_r {}", ninja_kb.sec_matrix);
    let mat:[&Matrix;2] =[&ninja_kb.matrix,&ninja_kb.sec_matrix];
    let mat_last:[&Matrix;2] =[&ninja_kb.matrix_last,&ninja_kb.sec_matrix_last];
    
    for side in 0..SIDES{
        for col in 0..COLS  {
            for row in 0..ROWS{
                //info!("side{} {}",side, mat[side][row][col]);
                //pressed        
                if mat[side][row][col] && !mat_last[side][row][col]{
                    //info!("pressed {} {}",row,col);
                    ninja_kb.led.set_low();
                    match ninja_kb.keys[side][ninja_kb.layer][row][col]{
                        Key::Layer=>{
                            ninja_kb.layer=1;
                            event=false;
                            continue;
                        },
                        Key::Code(code)=>{
                            //info!("code {}",code as u8);
                            let mut k=REPORT_BUFF_MAX;
                            let mut duplicate=false;
                            for i in 0..REPORT_BUFF_MAX{
                                if k==REPORT_BUFF_MAX && ninja_kb.report_buff[i]==Keyboard::NoEventIndicated {
                                    k=i;
                                }
                                if ninja_kb.report_buff[i]==code{
                                    duplicate=true;
                                    break;
                                }
                                /*if ninja_kb.report_buff[i]==Keyboard::NoEventIndicated{
                                    event=true;
                                    ninja_kb.report_buff[i]=code;
                                    break;
                                }*/
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
                if !mat[side][row][col] && mat_last[side][row][col]{
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
                                    ninja_kb.report_buff[i]=Keyboard::NoEventIndicated;
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

//enum Key{
//    Code(Keyboard),
//    Layer,
//    NoKey,
//}
fn serialize_key(key:&Key)->(u8,u8){
    match key{
        Key::Code(code)=>(0,*code as u8),
        Key::Layer=>(1,0),
        Key::NoKey=>(2,0)
    }
}
fn deserialize_key(b1:u8,b2:u8)->Key{
    match b1{
        0=> Key::Code(Keyboard::from(b2)),
        1=> Key::Layer,
        2=>Key::NoKey,
        _=>Key::NoKey
    }
}
fn serialize_keys(side:u8,layer:u8,side_data:&Side)->[u8;64]{
    let mut bytes:[u8;64]=[0;64];
    let mut i:usize=4;
    bytes[0]=1;//keys
    bytes[1]=0;//reserved
    bytes[2]=side;
    bytes[3]=layer;
    for row in 0..ROWS{
        for col in 0..COLS  {
            let k=serialize_key(&side_data[row][col]);
            if i+1 < 64{
                bytes[i  ]=k.0;
                bytes[i+1]=k.1;
            }
            i+=2;
        }
    }    
    bytes
}
fn deserialize_keys(bytes:&[u8;64],keys:&mut Keys){
    let side=bytes[2] as usize;
    let layer=bytes[3] as usize;                            
    let mut k:usize=4;
    for row in 0..ROWS{
        for col in 0..COLS  {
            let key=deserialize_key(bytes[k],bytes[k+1]);
            if side < SIDES && layer < LAYERS {
                keys[side][layer][row][col]=key;
            }
            if k+1 < 64{
                k+=2;
            }
        }
    }
}
fn write_conf_to_eeprom(keys:&mut Keys,eeprom:&mut EepromT,delay:u32){
    //let mut bytes:[u8;CONF_SIZE]=[255;CONF_SIZE];
    let mut bytes=[[0u8;PAGE_SIZE];CONF_PAGES];
    
    let mut i:usize=2;
    let mut p=0;
    bytes[p][0]=EEPROM_MARK;
    info!("writing.");
    for side in 0..SIDES{
        for layer in 0..LAYERS{
            for col in 0..COLS{
                for row in 0..ROWS{
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
    //info!("eeprom pages {}",bytes);
    let memory_address = 0u32;
    //info!("eeprom pages {}",CONF_PAGES);
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
            //info!("read bytes {}", bytes);
            let mut i:usize=2;
            if bytes[0]==EEPROM_MARK{
                for side in 0..SIDES{
                    for layer in 0..LAYERS{
                        for col in 0..COLS{
                            for row in 0..ROWS{
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