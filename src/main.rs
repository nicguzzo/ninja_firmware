#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use core::sync::atomic::{AtomicBool, Ordering};

use defmt::{panic, *};
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_futures::select::{select, Either};
use embassy_stm32::dma::NoDma;
use embassy_stm32::gpio::{Level, Output, Speed, Input, Pull, AnyPin,Pin};
use embassy_stm32::i2c::{I2c, TimeoutI2c, Error};
use embassy_stm32::pwm::Channel;
use embassy_stm32::pwm::simple_pwm::{SimplePwm,PwmPin};
use embassy_stm32::time::{Hertz, khz};
use embassy_stm32::usb::{Driver, Instance};
use embassy_stm32::{interrupt, Config};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;
use embassy_time::{Duration, Timer,Instant};
use embassy_usb::class::hid::{HidReaderWriter, ReportId, RequestHandler, State};
use embassy_usb::control::OutResponse;
use embassy_usb::{Builder, DeviceStateHandler};
use usbd_hid::descriptor::{KeyboardReport, SerializedDescriptor};

use {defmt_rtt as _, panic_probe as _};

const COLS:usize=6;
const ROWS:usize=4;
const LAYERS:usize=2;
//const DEBOUNCE:u64=5;

const SECONDARY_KB_ADDRESS: u8 = 0x08;
const WHOAMI: u8 = 0x0F;
const SECONDARY_KB_N_BYTES:usize = 3;

static SUSPENDED: AtomicBool = AtomicBool::new(false);
mod keys;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    
    //enable PB3 and PB4 as gpio
    unsafe{        
        embassy_stm32::pac::RCC.apb2enr().modify(|w| w.set_afioen(true));        
        embassy_stm32::pac::AFIO.mapr().modify(|w| w.set_swj_cfg(0b010));
    }
    let mut config = Config::default();
    config.rcc.hse = Some(Hertz(8_000_000));
    config.rcc.sys_ck = Some(Hertz(48_000_000));
    config.rcc.pclk1 = Some(Hertz(24_000_000));
    
    let mut p = embassy_stm32::init(config);
    
    let mut led = Output::new(p.PC13.degrade(), Level::High, Speed::Low);

    
    //RGB LED 
    //let ch_r = PwmPin::new_ch1(p.PA6);
    //let ch_g = PwmPin::new_ch2(p.PA7);
    //let ch_b = PwmPin::new_ch4(p.PB1);    
    //let mut led_rgb_pwm = SimplePwm::new(p.TIM3, Some(ch_r), Some(ch_g),None, Some(ch_b), khz(10));    
    //let max = led_rgb_pwm.get_max_duty();    
    //led_rgb_pwm.enable(Channel::Ch4);
    //led_rgb_pwm.set_duty(Channel::Ch4, 0);

    

    let row0 = Output::new(p.PB5.degrade(),Level::Low, Speed::Low); 
    let row1 = Output::new(p.PB6.degrade(),Level::Low, Speed::Low);
    let row2 = Output::new(p.PB7.degrade(),Level::Low, Speed::Low);
    let row3 = Output::new(p.PB8.degrade(),Level::Low, Speed::Low);

    let col0 = Input::new(p.PB12.degrade(),Pull::Down);
    let col1 = Input::new(p.PB13.degrade(),Pull::Down);
    let col2 = Input::new(p.PB14.degrade(),Pull::Down);
    let col3 = Input::new(p.PB15.degrade(),Pull::Down);
    let col4 = Input::new(p.PB3.degrade(), Pull::Down);
    let col5 = Input::new(p.PB4.degrade(), Pull::Down);

    //keyboard matrix
    let mut matrix:[[bool; COLS]; ROWS]=[ [false; COLS]; ROWS];    
    let mut matrix_last:[[bool; COLS]; ROWS]= [ [false; COLS]; ROWS];
    //let mut matrix_debounce:[[u64; COLS]; ROWS]= [ [0; COLS]; ROWS];
    
    let mut keys_right:[[[u8; COLS]; ROWS];LAYERS]=[
        [
            [keys::KEY_ESC, keys::KEY_Q,keys::KEY_W,keys::KEY_E,keys::KEY_R,keys::KEY_T ],
            [keys::KEY_TAB, keys::KEY_A,keys::KEY_S,keys::KEY_D,keys::KEY_F,keys::KEY_G ],
            [keys::KEY_TAB, keys::KEY_Z,keys::KEY_X,keys::KEY_C,keys::KEY_V,keys::KEY_B ],
            [0,0,0         ,keys::KEY_RIGHTCTRL,keys::KEY_RIGHTSHIFT,keys::KEY_RIGHTALT ],
        ],
        [
            [keys::KEY_ESC, keys::KEY_Q,keys::KEY_W,keys::KEY_E,keys::KEY_R,keys::KEY_T ],
            [keys::KEY_TAB, keys::KEY_A,keys::KEY_S,keys::KEY_D,keys::KEY_F,keys::KEY_G ],
            [keys::KEY_TAB, keys::KEY_Z,keys::KEY_X,keys::KEY_C,keys::KEY_V,keys::KEY_B ],
            [0,0,0         ,keys::KEY_RIGHTCTRL,keys::KEY_RIGHTSHIFT,keys::KEY_RIGHTALT ],
        ]
    ];

    let mut keys_left:[[[u8; COLS]; ROWS]; LAYERS]=[
        [
            [keys::KEY_Y, keys::KEY_U,keys::KEY_I,keys::KEY_O,keys::KEY_P,keys::KEY_BACKSPACE ],
            [keys::KEY_H, keys::KEY_J,keys::KEY_K,keys::KEY_L,keys::KEY_F,keys::KEY_ENTER ],
            [keys::KEY_N, keys::KEY_M,keys::KEY_X,keys::KEY_LEFTBRACE,keys::KEY_RIGHTBRACE,keys::KEY_APOSTROPHE ],
            [keys::KEY_LAYER,keys::KEY_DOT,keys::KEY_SPACE ,0,0,0]
        ],
        [
            [keys::KEY_Y, keys::KEY_U,keys::KEY_I,keys::KEY_O,keys::KEY_P,keys::KEY_BACKSPACE ],
            [keys::KEY_H, keys::KEY_UP,keys::KEY_K,keys::KEY_L,keys::KEY_F,keys::KEY_ENTER ],
            [keys::KEY_LEFT, keys::KEY_DOWN,keys::KEY_RIGHT,keys::KEY_LEFTBRACE,keys::KEY_RIGHTBRACE,keys::KEY_APOSTROPHE ],
            [keys::KEY_LAYER,keys::KEY_DOT,keys::KEY_SPACE ,0,0,0]
        ]
    ];
    let mut layer:usize=0;
    let mut rows:[Output<'static,AnyPin>; ROWS]=[row0,row1,row2,row3];
    let cols:[Input <'static,AnyPin>; COLS]=[col0,col1,col2,col3,col4,col5];

    {
        // BluePill board has a pull-up resistor on the D+ line.
        // Pull the D+ pin down to send a RESET condition to the USB bus.
        // This forced reset is needed only for development, without it host
        // will not reset your device when you upload new firmware.
        let _dp = Output::new(&mut p.PA12, Level::Low, Speed::Low);
        Timer::after(Duration::from_millis(100)).await;
    }

    let irq = interrupt::take!(I2C2_EV);    
    let mut i2c = I2c::new(p.I2C2, p.PB10, p.PB11,irq,NoDma,NoDma,Hertz(100_000),Default::default());    
    let mut timeout_i2c = TimeoutI2c::new(&mut i2c, Duration::from_millis(50));
    let mut i2c_data_send = [0u8; 1];
    let mut i2c_data_recv = [0u8; SECONDARY_KB_N_BYTES];
    

    let mut sec_matrix:[[bool; COLS]; ROWS]=[ [false; COLS]; ROWS];    
    let mut sec_matrix_last:[[bool; COLS]; ROWS]= [ [false; COLS]; ROWS];
    
     // Create the driver, from the HAL.
    let irq = interrupt::take!(USB_LP_CAN1_RX0);
    let driver = Driver::new(p.USB, irq, p.PA12, p.PA11);

    let mut config =  embassy_usb::Config::new(0xc0de, 0xcafe);
    config.manufacturer = Some("Nicguzzo");
    config.product = Some("Ninja corne");
    config.serial_number = Some("12345678");
    config.max_power = 500;
    config.max_packet_size_0 = 64;
    config.supports_remote_wakeup = true;

    let mut device_descriptor = [0; 256];
    let mut config_descriptor = [0; 256];
    let mut bos_descriptor = [0; 256];
    let mut control_buf = [0; 64];
    let request_handler = MyRequestHandler {};
    let device_state_handler = MyDeviceStateHandler::new();

    let mut state = State::new();

    let mut builder = Builder::new(
        driver,
        config,
        &mut device_descriptor,
        &mut config_descriptor,
        &mut bos_descriptor,
        &mut control_buf,
        Some(&device_state_handler),
    );

    // Create classes on the builder.
    let config = embassy_usb::class::hid::Config {
        report_descriptor: KeyboardReport::desc(),
        request_handler: Some(&request_handler),
        poll_ms: 60,
        max_packet_size: 64,
    };
    let hid = HidReaderWriter::<_, 1, 8>::new(&mut builder, &mut state, config);

    // Build the builder.
    let mut usb = builder.build();

    let remote_wakeup: Signal<CriticalSectionRawMutex, _> = Signal::new();

    // Run the USB device.
    let usb_fut = async {
        loop {
            usb.run_until_suspend().await;
            match select(usb.wait_resume(), remote_wakeup.wait()).await {
                Either::First(_) => (),
                Either::Second(_) => unwrap!(usb.remote_wakeup().await),
            }
        }
    };

    let (reader, mut writer) = hid.split();

    // Do stuff with the class!
    let in_fut = async {
        //const NKRO:usize=21*2; //corne
        //let mut report_nkro:[u8;NKRO]=[0;NKRO];
        //limited to 6 keys, for now
        let report_buff_max=6;
        let mut report_lim6:[u8;6]=[0;6];
        //let mut instant= Instant::now();
        loop {
            for row in 0..ROWS{
                rows[row].set_high();
                for col in 0..COLS  {
                    matrix_last[row][col]=matrix[row][col];
                    matrix[row][col]=cols[col].is_high();
                    
                }
                rows[row].set_low();
            }
            //read i2c sync
            let res=timeout_i2c.blocking_write_read(SECONDARY_KB_ADDRESS, &i2c_data_send,&mut i2c_data_recv);
            match res{
                Ok(_) => {
                    /*info!("sec keys \n{:08b}\n{:08b}\n{:08b}\n",
                    i2c_data_recv[0],
                    i2c_data_recv[1],
                    i2c_data_recv[2]
                    );*/
                    let mut b=0;
                    let mut k:usize=0;
                    let mut bit:u8=7;
                    for col in 0..COLS  {
                        for row in 0..ROWS{
                            sec_matrix_last[row][col]=sec_matrix[row][col];
                            b=k/8;
                            sec_matrix[row][col]= ((i2c_data_recv[b]>>bit)&0x01) != 0;
                            if bit==0{
                                bit=7;
                            }else{
                                bit=bit-1;
                            }
                            k+=1;
                        }
                    }
                },
                Err(Error::Timeout) => {
                    led.set_low();
                    error!("Operation timed out");
                },
                Err(e) => error!("I2c write Error: {:?}", e),                    
            }
            
            let mut event=false;
            
            for row in 0..ROWS{                
                for col in 0..COLS  {
                    //pressed                    
                    if matrix[row][col] && !matrix_last[row][col]{                        
                        if keys_right[layer][row][col]== keys::KEY_LAYER{                            
                            layer=1;
                            info!("layer {}",layer);
                            event=false;
                            continue;
                        }
                        
                        for i in 0..report_buff_max{
                            if report_lim6[i]==0{
                                event=true;
                                report_lim6[i]=keys_right[layer][row][col];
                                break;
                            }
                        }
                    }
                    if sec_matrix[row][col] && !sec_matrix_last[row][col]{                        
                        if keys_left[layer][row][col]== keys::KEY_LAYER{                            
                            layer=1;
                            info!("layer {}",layer);
                            event=false;
                            continue;
                        }
                        for i in 0..report_buff_max{
                            if report_lim6[i]==0{
                                event=true;
                                report_lim6[i]=keys_left[layer][row][col];
                                break;
                            }
                        }
                    }
                    //released
                    if !matrix[row][col] && matrix_last[row][col]{
                        if keys_right[layer][row][col]== keys::KEY_LAYER {
                            layer=0;
                            info!("layer {}",layer);
                            event=false;
                            continue;
                        }
                        for i in 0..report_buff_max{
                            if report_lim6[i]==keys_right[layer][row][col]{
                                event=true;
                                report_lim6[i]=0;
                                break;
                            }
                        }                        
                    }
                    if !sec_matrix[row][col] && sec_matrix_last[row][col]{
                        if keys_left[layer][row][col]== keys::KEY_LAYER {
                            layer=0;
                            info!("layer {}",layer);
                            event=false;
                            continue;
                        }
                        for i in 0..report_buff_max{
                            if report_lim6[i]==keys_left[layer][row][col]{
                                event=true;
                                report_lim6[i]=0;
                                break;
                            }
                        }                        
                    }
                }
            }
            if SUSPENDED.load(Ordering::Acquire) {
                info!("Triggering remote wakeup");
                remote_wakeup.signal(());
            }else{
                if event {
                    //led.set_low();
                    let report = KeyboardReport {
                        keycodes: report_lim6,
                        leds: 0,
                        modifier: 0,
                        reserved: 0,
                    };
                    match writer.write_serialize(&report).await {
                        Ok(()) => {}
                        Err(e) => warn!("Failed to send report: {:?}", e),
                    };
                }
            }
            Timer::after(Duration::from_millis(1)).await;
        }
    };

    let out_fut = async {
        reader.run(false, &request_handler).await;
    };
    
    join(usb_fut, join(in_fut, out_fut)).await;
    
}

struct MyRequestHandler {}

impl RequestHandler for MyRequestHandler {
    fn get_report(&self, id: ReportId, _buf: &mut [u8]) -> Option<usize> {
        info!("Get report for {:?}", id);
        None
    }

    fn set_report(&self, id: ReportId, data: &[u8]) -> OutResponse {
        info!("Set report for {:?}: {=[u8]}", id, data);
        OutResponse::Accepted
    }

    fn set_idle_ms(&self, id: Option<ReportId>, dur: u32) {
        info!("Set idle rate for {:?} to {:?}", id, dur);
    }

    fn get_idle_ms(&self, id: Option<ReportId>) -> Option<u32> {
        info!("Get idle rate for {:?}", id);
        None
    }
}

struct MyDeviceStateHandler {
    configured: AtomicBool,
}

impl MyDeviceStateHandler {
    fn new() -> Self {
        MyDeviceStateHandler {
            configured: AtomicBool::new(false),
        }
    }
}

impl DeviceStateHandler for MyDeviceStateHandler {
    fn enabled(&self, enabled: bool) {
        self.configured.store(false, Ordering::Relaxed);
        SUSPENDED.store(false, Ordering::Release);
        if enabled {
            info!("Device enabled");
        } else {
            info!("Device disabled");
        }
    }

    fn reset(&self) {
        self.configured.store(false, Ordering::Relaxed);
        info!("Bus reset, the Vbus current limit is 100mA");
    }

    fn addressed(&self, addr: u8) {
        self.configured.store(false, Ordering::Relaxed);
        info!("USB address set to: {}", addr);
    }

    fn configured(&self, configured: bool) {
        self.configured.store(configured, Ordering::Relaxed);
        if configured {
            info!("Device configured, it may now draw up to the configured current limit from Vbus.")
        } else {
            info!("Device is no longer configured, the Vbus current limit is 100mA.");
        }
    }

    fn suspended(&self, suspended: bool) {
        if suspended {
            info!("Device suspended, the Vbus current limit is 500µA (or 2.5mA for high-power devices with remote wakeup enabled).");
            SUSPENDED.store(true, Ordering::Release);
        } else {
            SUSPENDED.store(false, Ordering::Release);
            if self.configured.load(Ordering::Relaxed) {
                info!("Device resumed, it may now draw up to the configured current limit from Vbus");
            } else {
                info!("Device resumed, the Vbus current limit is 100mA");
            }
        }
    }
}
