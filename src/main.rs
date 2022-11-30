#![no_main]
#![no_std]

use defmt_rtt as _; 
// global logger
use panic_probe as _;



#[defmt::panic_handler]
fn panic() -> ! {
    cortex_m::asm::udf()
}
use frunk::{HCons, HNil};    
use stm32f1xx_hal::device::I2C2;
use stm32f1xx_hal::gpio::{ErasedPin,Input,PushPull, PullUp, Output, Alternate, OpenDrain, PB10, PB11};
use stm32f1xx_hal::{prelude::*, gpio::PinState};
use stm32f1xx_hal::usb::{UsbBus, UsbBusType};
use stm32f1xx_hal::{pac, usb::Peripheral};
//use stm32f1xx_hal::flash::FlashWriter;
//use cortex_m::peripheral::NVIC;
use stm32f1xx_hal::{timer::{Event,CounterUs, SysDelay}, device::Interrupt, i2c::{BlockingI2c, Mode, DutyCycle}};
use usb_device::class_prelude::*;
use usb_device::prelude::*;
use usbd_human_interface_device::device::consumer::{MultipleConsumerReport, ConsumerControlInterface};
use usbd_human_interface_device::device::mouse::{WheelMouseReport, WheelMouseInterface};
use usbd_human_interface_device::page::{Keyboard,Consumer};
use usbd_human_interface_device::device::keyboard::{KeyboardLedsReport, NKROBootKeyboardInterface};
use usbd_human_interface_device::prelude::*;
use defmt::{info};
//use cortex_m::asm::delay;
mod config_class;
use config_class::RawConfInterface;

const REPORT_BUFF_MAX:usize=42;
const COLS:usize=6;
const ROWS:usize=4;
const LAYERS:usize=2;
const SIDES:usize=2;
const SECONDARY_KB_ADDRESS: u8 = 0x08;
const SECONDARY_KB_N_BYTES:usize = 3;
const CONF_SIZE:usize=COLS*ROWS*LAYERS*SIDES*2;
//type UsbBusT<'a> = UsbBusAllocator<UsbBus<Peripheral>>;
type UsbDev<'a>  = UsbDevice<'a, UsbBus<Peripheral>>;

type UsbKb<'a> =
UsbHidClass<
    UsbBus<Peripheral>, 
        HCons<RawConfInterface<'a, UsbBus<Peripheral>>, 
        HCons<ConsumerControlInterface<'a, UsbBus<Peripheral>>, 
        HCons<WheelMouseInterface<'a, UsbBus<Peripheral>>, 
        HCons<NKROBootKeyboardInterface<'a,UsbBus<Peripheral>>, HNil>>>>>;


type I2cT=stm32f1xx_hal::i2c::blocking::BlockingI2c::<I2C2, (PB10<Alternate<OpenDrain>>, PB11<Alternate<OpenDrain>>)>;
type Matrix= [[bool; COLS]; ROWS];

type Rows = [ErasedPin<Input<PullUp>>; ROWS];
type Cols = [ErasedPin<Output<PushPull>>; COLS];

type ReportBuff=[Keyboard;REPORT_BUFF_MAX];
enum Key{
    Code(Keyboard),
    Layer,
    NoKey,
}
type Side = [[Key; COLS]; ROWS];

struct Keys{
    left:[Side;LAYERS],
    right:[Side;LAYERS]
}
pub struct NinjaKb{
    rows:Rows,
    cols:Cols,
    matrix:Matrix,    
    matrix_last:Matrix,
    sec_matrix:Matrix,
    sec_matrix_last:Matrix,
    keys:Keys,
    report_buff:ReportBuff,
    
    layer:usize,
    led:ErasedPin<Output<PushPull>>
}

#[rtic::app(device = stm32f1xx_hal::pac)]
mod app {
    use crate::*;
    #[shared]
    struct Shared {        
        usb_dev: UsbDev<'static>,
        hid_kb: UsbKb<'static>,
        //ninja_kb:NinjaKb
    }

    #[local]
    struct Local {
        //usb_bus: UsbBusT<'static>,
        timer:CounterUs<pac::TIM2>,
        ninja_kb:NinjaKb,
        i2c:I2cT
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
    
        
        let mut keys=Keys{
            left:[
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
                ]
            ],
            right:[
                [
                    [Key::Code(Keyboard::Y), Key::Code(Keyboard::U),Key::Code(Keyboard::I),Key::Code(Keyboard::O),Key::Code(Keyboard::P),Key::Code(Keyboard::DeleteBackspace) ],
                    [Key::Code(Keyboard::H), Key::Code(Keyboard::J),Key::Code(Keyboard::K),Key::Code(Keyboard::L),Key::Code(Keyboard::F),Key::Code(Keyboard::ReturnEnter) ],
                    [Key::Code(Keyboard::N), Key::Code(Keyboard::M),Key::Code(Keyboard::X),Key::Code(Keyboard::LeftBrace),Key::Code(Keyboard::RightBrace),Key::Code(Keyboard::Apostrophe) ],
                    [Key::Code(Keyboard::ReturnEnter),Key::Code(Keyboard::Space),Key::Code(Keyboard::Dot) ,Key::NoKey,Key::NoKey,Key::NoKey]
                ],
                [
                    [Key::Code(Keyboard::F7)    ,Key::Code(Keyboard::F8)  ,Key::Code(Keyboard::F9)    ,Key::Code(Keyboard::F10)   ,Key::Code(Keyboard::F11)     ,Key::Code(Keyboard::F12) ],
                    [Key::Code(Keyboard::Keyboard6),Key::Code(Keyboard::UpArrow)  ,Key::Code(Keyboard::Keyboard7),Key::Code(Keyboard::Keyboard8),Key::Code(Keyboard::Keyboard9),Key::Code(Keyboard::Keyboard0)],
                    [Key::Code(Keyboard::LeftArrow)  ,Key::Code(Keyboard::DownArrow),Key::Code(Keyboard::Equal) ,Key::Code(Keyboard::PageUp),Key::Code(Keyboard::PageDown),Key::Code(Keyboard::Minus) ],
                    [Key::Code(Keyboard::DeleteForward),Key::Code(Keyboard::Home),Key::Code(Keyboard::End)   ,Key::NoKey           ,Key::NoKey              ,Key::NoKey]
                ]
            ]
        };
        let report_buff:ReportBuff = [Keyboard::NoEventIndicated;REPORT_BUFF_MAX];
        
        
        
        info!("size {}",CONF_SIZE);
        
        /*let mut flash_writer=flash.writer(stm32f1xx_hal::flash::SectorSize::Sz1K, stm32f1xx_hal::flash::FlashSize::Sz64K);
        let offset=1024*63;
        let mark=flash_writer.read(offset, CONF_SIZE);
        match mark{
            Ok(read_bytes)=>{
                info!("conf in flash {}",read_bytes);
                if read_bytes[0]==255{ //no flash data
                    let mut bytes:[u8;CONF_SIZE]=[255;CONF_SIZE];
                    let mut i:usize=0;
                    info!("writing.");
                    for l in 0..LAYERS{
                        for col in 0..COLS  {
                            for row in 0..ROWS{
                                let k=serialize_key(&keys.left[l][row][col]);
                                bytes[i  ]=k.0;
                                bytes[i+1]=k.1;
                                i+=2;
                            }
                        }
                    }
                    for l in 0..LAYERS{
                        for col in 0..COLS  {
                            for row in 0..ROWS{
                                let k=serialize_key(&keys.right[l][row][col]);
                                bytes[i  ]=k.0;
                                bytes[i+1]=k.1;
                                i+=2;
                            }
                        }
                    }
                    info!("bytes{} {}",bytes,i);
                    match flash_writer.write(offset, &bytes){
                        Ok(_)=>info!("conf written."),
                        Err(_e)=>info!("error writing conf"),
                    }
                }else{//read flash to conf
                    let mut i=0;
                    info!("reading.");
                    for l in 0..LAYERS{
                        for col in 0..COLS  {
                            for row in 0..ROWS{
                                keys.left[l][row][col]=deserialize_key(read_bytes[i],read_bytes[i+1]);
                                i+=2;
                            }
                        }
                    }
                    for l in 0..LAYERS{
                        for col in 0..COLS  {
                            for row in 0..ROWS{
                                keys.right[l][row][col]=deserialize_key(read_bytes[i],read_bytes[i+1]);
                                i+=2;
                            }
                        }
                    }
                    info!("conf read.");
                }
            },
            _=>info!("error reading mark from flash")
        }*/
        
        
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
    
        let mut i2c = BlockingI2c::i2c2(
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
        let bytes :[u8;1]=[0;1];
        let mut buffer:[u8;SECONDARY_KB_N_BYTES]=[0;SECONDARY_KB_N_BYTES];
        match i2c.write_read(SECONDARY_KB_ADDRESS, &bytes, &mut buffer){
            Ok(_) =>{
                led.set_low();
            },
            Err(_) =>{
                info!("i2c read/write error")
            }
        }
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
            led
        };
        (Shared {  usb_dev, hid_kb }, Local {timer,ninja_kb,i2c}, init::Monotonics())
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
    #[task(binds = TIM2, priority = 3, shared = [hid_kb], local=[timer, ninja_kb,i2c])]
    fn tick(cx: tick::Context) {
        let mut hid_kb = cx.shared.hid_kb;
        //cx.local.ninja_kb.led.set_low();
        (&mut hid_kb).lock(|hid_kb| {
            let keyboard = hid_kb.interface::<NKROBootKeyboardInterface<'_, _>, _>();
            //let control = hid_kb.interface::<ConsumerControlInterface<'_, _>, _>();
            if ninja(cx.local.ninja_kb,cx.local.i2c)
            {
                match keyboard.write_report(cx.local.ninja_kb.report_buff) {
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
                    info!("read conf report {}",s.packet)
                }
                Err(_e) => {
                    info!("Failed to read conf report: ")
                }
            }
            
        });
        cx.local.timer.clear_interrupt(Event::Update);
    }
    #[idle()]
    fn idle(_cx: idle::Context) -> ! {
        info!("idle");
        loop {
            cortex_m::asm::nop();
        }
    }    
}

fn usb_poll(usb_dev: &mut UsbDev, keyboard: &mut UsbKb) {
    if usb_dev.poll(&mut [keyboard]) {
        keyboard.poll();
    }
}


fn ninja(ninja_kb:&mut NinjaKb,i2c:&mut I2cT)-> bool{
    let mut event=false;
    for col in 0..COLS  {
        ninja_kb.cols[col].set_low();
        for row in 0..ROWS {
            ninja_kb.matrix_last[row][col]=ninja_kb.matrix[row][col];
            ninja_kb.matrix[row][col]=ninja_kb.rows[row].is_low();
        }
        ninja_kb.cols[col].set_high();
    }
    
    let bytes :[u8;1]=[0;1];
    let mut buffer:[u8;SECONDARY_KB_N_BYTES]=[0;SECONDARY_KB_N_BYTES];
    match i2c.write_read(SECONDARY_KB_ADDRESS, &bytes, &mut buffer){
        Ok(_) =>{
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

    //trace!("matrix_l {}", ninja_kb.matrix);
    //trace!("matrix_r {}", ninja_kb.sec_matrix);
    let mat:[&Matrix;2] =[&ninja_kb.matrix,&ninja_kb.sec_matrix];
    let mat_last:[&Matrix;2] =[&ninja_kb.matrix_last,&ninja_kb.sec_matrix_last];
    let side:[&Side;2]=[&ninja_kb.keys.left[ninja_kb.layer],&ninja_kb.keys.right[ninja_kb.layer]];
    for m in 0..LAYERS{
        for col in 0..COLS  {
            for row in 0..ROWS{
                //pressed        
                if mat[m][row][col] && !mat_last[m][row][col]{      
                    ninja_kb.led.set_low();
                    match side[m][row][col]{
                        Key::Layer=>{
                            ninja_kb.layer=1;
                            event=false;
                            continue;
                        },
                        Key::Code(code)=>{
                            //info!("code {:08b}",code as u8);
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
                if !mat[m][row][col] && mat_last[m][row][col]{
                    ninja_kb.led.set_high();
                    match side[m][row][col]{
                        Key::Layer=>{
                            ninja_kb.layer=1;
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
/*fn serialize_keys(all_keys:&Keys)->[u8;CONF_SIZE]{
    let mut out:[u8;CONF_SIZE]=[0;CONF_SIZE];
    let mut i=0;
    let sides=[&all_keys.left,&all_keys.right];
    for side in sides {
        for layer in side{
            for keys in layer{
                for key in keys{
                    let k=serialize_key(key);
                    if i < CONF_SIZE{
                        out[i]=k.0;
                        out[i+1]=k.1;
                        i+=2;
                    }
                }
            }
        }
    }
    out
}*/
