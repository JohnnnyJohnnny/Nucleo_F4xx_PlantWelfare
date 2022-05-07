#![no_main]
#![no_std]
#![feature(alloc_error_handler)]

use core::cell::RefCell;
use core::sync::atomic::{AtomicBool, Ordering};

use cortex_m::peripheral::Peripherals;
use cortex_m::interrupt::Mutex;
use cortex_m_rt::entry;
use panic_rtt_target as _;
use rtt_target::{rtt_init_print, rprintln};


extern crate alloc;

use alloc::format;
use alloc_cortex_m::CortexMHeap;
use core::alloc::Layout;
use core::convert::TryInto;
use Nucleo_F4xx_PlantWelfare::{
    hal::{
		gpio::Edge, interrupt,
		pac,
		//delay::Delay, 
		rtc::Rtc,
		adc::{Adc, config::AdcConfig, config::SampleTime}, 
        serial::{config::Config, Serial},
        //stm32::RTC, stm32::PWR,
		prelude::*
		},
    //pac, 
	Button, Led, 
    Watering, SerialInterface,SerialCommand,
};

//use rtcc::{NaiveDate, NaiveDateTime, NaiveTime, Rtcc};
use time::{
    macros::{date, time}, 
	Time, PrimitiveDateTime,
};
//use time::Duration;

// Used to signal to the main loop that it should toggle the led
static SIGNAL: AtomicBool = AtomicBool::new(false);

static BUTTON: Mutex<RefCell<Option<Button>>> = Mutex::new(RefCell::new(None));

#[global_allocator]
static ALLOCATOR: CortexMHeap = CortexMHeap::empty();


#[entry]
fn main() -> ! {	
	// this is the initalization part and only that
	// the ST-Link resets the device if the USB is plugged in (yes, it's powered over an external 5V supply)
	// so settings should be applied over the serial interface
    rtt_init_print!();

    let mut p = pac::Peripherals::take().unwrap();
    let cp = Peripherals::take().unwrap();

    let start = cortex_m_rt::heap_start() as usize;
    let size = 1024; // in bytes
    unsafe { ALLOCATOR.init(start, size) }

	let gpioa = p.GPIOA.split();
	let gpiob = p.GPIOB.split();
	let gpioc = p.GPIOC.split();

    // (Re-)configure PA5 (LD2 - User Led) as output
    let mut led = Led::new(gpioa.pa5.into_push_pull_output());
    led.set(false);

	// iniialize Relais logic
	let mut water = Watering::new(	gpiob.pb3.into_push_pull_output(),
								 	gpiob.pb5.into_push_pull_output(), 
									gpiob.pb4.into_push_pull_output(),
								  	gpiob.pb10.into_push_pull_output(), 
									gpioa.pa8.into_push_pull_output(), 
									gpioc.pc4.into_push_pull_output());
	
    // Constrain clock registers
    let rcc = p.RCC.constrain();

    let clocks = rcc.cfgr.sysclk(84.MHz()).freeze();

	let mut syscfg = p.SYSCFG.constrain();
    
    let tx_pin = gpioa.pa2.into_alternate();
    let rx_pin = gpioa.pa3.into_alternate();
    let config = Config::default().baudrate(115_200.bps());
	let serial = p.USART2.serial((tx_pin, rx_pin), config, &clocks).unwrap();
    
    let (mut tx, mut rx) = serial.split();
    let mut serialprint = SerialInterface::new(tx, rx); 
	serialprint.init();

	let mut rtc = Rtc::new(p.RTC, &mut p.PWR);
	//rtc.set_24h_fmt();

    // Get delay provider
    let mut delay = p.TIM5.delay_us(&clocks);

	// Configure PC5 (User B1) as an input and enable external interrupt
    let mut button = Button::new(gpioc.pc13);
    button.enable_interrupt(Edge::Rising, &mut syscfg, &mut p.EXTI);

    cortex_m::interrupt::free(|cs| {
        BUTTON.borrow(cs).replace(Some(button));
    });

    // Enable the external interrupt todo check what is wrong with the Interrupts 
    // unsafe 
	// {
    //     cortex_m::peripheral::NVIC::unmask(pac::Interrupt::EXTI15_10.into());
    // }

	// new part -> initilize the ADC 
	let mut adc = Adc::adc1(p.ADC1, true, AdcConfig::default());
	let pa0 = gpioa.pa0.into_analog();
	let pa1 = gpioa.pa1.into_analog();
	let pa4 = gpioa.pa4.into_analog();
	let pb0 = gpiob.pb0.into_analog();
	let pc1 = gpioc.pc1.into_analog();
	let pc0 = gpioc.pc0.into_analog();

	//let wateringtime : NaiveTime = NaiveTime::from_hms_milli(18, 00, 00, 0); // at 18:00 we start the pumps 
	let wateringtime : Time = time!(18:00); // at 18:00 we start the pumps 
	
	//let mut today = NaiveDate::from_ymd(2021, 1, 1); // fallback if 5V blackout 
	let mut today = date!(2021 - 1 - 1);
	today = rtc.get_datetime().date();
	let mut nextWaterInterval : PrimitiveDateTime = PrimitiveDateTime::new(today.next_day().unwrap(),  wateringtime); 
	let mut state = 0; // idle -> todo change to enum
	let mut waterlevel : [u16; 6] = [0,0,0,0,0,0];
	let mut relaistest = 9;

    loop 
	{
		let plantsample : [u16; 6] = [
			adc.convert(&pa0, SampleTime::Cycles_480),
			adc.convert(&pa1, SampleTime::Cycles_480),
			adc.convert(&pa4, SampleTime::Cycles_480),
			adc.convert(&pb0, SampleTime::Cycles_480),
			adc.convert(&pc1, SampleTime::Cycles_480),
			adc.convert(&pc0, SampleTime::Cycles_480)
		];

		let state_change = SIGNAL.load(Ordering::SeqCst);
        if state_change {
			if relaistest > 5 {
				relaistest = 0;
			} else {
				relaistest += 1;
			}
			water.relais_test(relaistest);

            SIGNAL.store(false, Ordering::SeqCst);
        }

		rprintln!("Plant{}: {}mV", 0, waterlevel[0]);
		//water.check_water_level(waterlevel);
		let log_date_time = format!("DateTime {}", rtc.get_datetime());
		rprintln!("{}", log_date_time);
        serialprint.printcycle();

		let mut time = rtc.get_datetime();

		if state == 0 {
			let dt_result = rtc.get_datetime();
			if dt_result >= nextWaterInterval 
			{
				rprintln!("NOW");
				state = 1;
				relaistest = 0;
				nextWaterInterval = PrimitiveDateTime::new(nextWaterInterval.date().next_day().unwrap(), nextWaterInterval.time());
			} 
			else 
			{
				let date = format!("{:?}",nextWaterInterval.date());
				rprintln!("wait {}",date );
			}
		}
		if state == 1 
		{
			water.relais_test(relaistest);			
	        delay.delay_ms(2000_u16);
			relaistest += 1;
			if relaistest > 6 {
				state = 0;
			}
		}

		let cmd = serialprint.check_for_command();
		match cmd	
		{
			SerialCommand::Empty => {},
			SerialCommand::PrintTime => serialprint.print_time(time),
			SerialCommand::PrintNext => {
											let date = format!("{:?}",nextWaterInterval.date());
											serialprint.printmessage(&date);
										},
			SerialCommand::ActivatePump(_idx) => {},
			SerialCommand::SetDate(_date) => 	{	
													rtc.set_date(&_date);
													nextWaterInterval = PrimitiveDateTime::new(_date, wateringtime); 
												},
			SerialCommand::SetNext(_date) => nextWaterInterval = PrimitiveDateTime::new(_date, wateringtime),
			SerialCommand::SetTime(_time) => {rtc.set_time(&_time);},
			SerialCommand::SetPlantConfig(_idx, power) => {},
			SerialCommand::SetPlantWateringDuration(_idx, _duration) => water.set_duration(_idx.try_into().unwrap(), _duration.try_into().unwrap()),
			SerialCommand::PrintUsage => serialprint.printcommands(),
		}
	
        delay.delay_ms(500_u16);
        led.toggle();
    }
}

#[alloc_error_handler]
fn oom(_: Layout) -> ! {
    loop {}
}

#[interrupt]
fn EXTI15_10() {
    // Clear the interrupt
    cortex_m::interrupt::free(|cs| {
        let mut button = BUTTON.borrow(cs).borrow_mut();
        button.as_mut().unwrap().clear_interrupt_pending_bit();
    });

    SIGNAL.store(true, Ordering::SeqCst);
}
