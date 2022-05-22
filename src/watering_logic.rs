/**
 * @todo	new Interface 
 * 			add start -> sets Tasks for all Plants
 * 			cycling(rtc time) -> starts pump and watches if confirued time is elapsed
 * 			task -> a watering task contains a Pump interval (time) or a moisture level
 */
use stm32f4xx_hal::gpio::{gpiob::PB3, gpiob::PB5, gpiob::PB4, gpiob::PB10,
						  gpioa::PA8, gpioc::PC4, Output, PushPull};

use embedded_hal::digital::v2::{OutputPin};
use time::{
    macros::time,
	Time, Duration,
	PrimitiveDateTime};
use rtt_target::{rprintln};

pub enum TaskState 
{
	Off, 
	WaitActivation, 
	Active,
}

struct WaterTask
{
	active 	 : TaskState,
	duration : u16,
	end 	 : Time,
}

pub struct WaterConfig
{
	duration : u16, 
	levellow : u16,
	levelhigh: u16,
}

impl WaterConfig
{
	pub fn new() -> WaterConfig
	{
		WaterConfig
		{
			duration : 1000,
			levellow : 1300,
			levelhigh : 1800,
		}
	}

	pub fn new_ex(dur : u16, low : u16, high : u16) -> WaterConfig
	{
		WaterConfig
		{
			duration : dur,
			levellow : low,
			levelhigh : high,
		}
	}

	pub fn to_array(&mut self) -> [u8; 6]
    {
		return [
			((self.duration >> 8) & 0xFF) as u8,
			((self.duration >> 0) & 0xFF) as u8,
			((self.levellow >> 8) & 0xFF) as u8,
			((self.levellow >> 0) & 0xFF) as u8,
			((self.levelhigh >> 8) & 0xFF) as u8,
			((self.levelhigh >> 0) & 0xFF) as u8,
		]
	}
}

const max_number_of_plants : usize = 6;

pub type PlantData = [WaterConfig; max_number_of_plants];


enum Relaisstate {On, Off}
/// Relais Pin 
pub struct Watering 
{
	relaisstate : Relaisstate,
	count : u32,
	relais1: PB3<Output<PushPull>>,
	relais2: PB5<Output<PushPull>>,
	relais3: PB4<Output<PushPull>>,
	relais4: PB10<Output<PushPull>>, 
	relais5: PA8<Output<PushPull>>,
	relais6: PC4<Output<PushPull>>,

	plantconfig : [WaterConfig; 6],
	task 		: [WaterTask; 6],		// things that currently have to be proceessed 
}

impl Watering {
	/**
	 * @param[in]  pin 	GPIO Pins which are connectet to the FETs  
	 * @detail     In don't know how the argument passing works. I have 6 FETs in Hardware, 
     *             they should be passed as array
	 * 
	 */
	pub fn new( pin1: PB3<Output<PushPull>>, pin2: PB5<Output<PushPull>>, pin3: PB4<Output<PushPull>>,
				   pin4: PB10<Output<PushPull>>, pin5: PA8<Output<PushPull>>, pin6: PC4<Output<PushPull>>,) -> Self 
	{
		let relais1 = pin1;	
		let relais2 = pin2;
		let relais3 = pin3;	
		let relais4 = pin4;
		let relais5 = pin5;	
		let relais6 = pin6;
		let relaisstate = Relaisstate::Off; 
		let count = 0;		
		let plantconfig :  [WaterConfig; 6] = [
			WaterConfig { duration : 6000, levellow : 2100, levelhigh: 1500,}, 
			WaterConfig { duration : 6000, levellow : 2100, levelhigh: 1500,}, 
			WaterConfig { duration : 6000, levellow : 2100, levelhigh: 1500,},
			WaterConfig { duration : 6000, levellow : 2100, levelhigh: 1500,},
			WaterConfig { duration : 6000, levellow : 2100, levelhigh: 1500,},
			WaterConfig { duration : 6000, levellow : 2100, levelhigh: 1500,}
			];
		let task : [WaterTask; 6] = [
			WaterTask { active : TaskState::Off, duration :0, end: time!(0:00), },
			WaterTask { active : TaskState::Off, duration :0, end: time!(0:00), },
			WaterTask { active : TaskState::Off, duration :0, end: time!(0:00), },
			WaterTask { active : TaskState::Off, duration :0, end: time!(0:00), },
			WaterTask { active : TaskState::Off, duration :0, end: time!(0:00), },
			WaterTask { active : TaskState::Off, duration :0, end: time!(0:00), },
			];

		Self {relais1, relais2, relais3, relais4, relais5, relais6, relaisstate, count,	plantconfig, task	}
	}

	/**
	 * @detail	add tasks for all plants based on config 
	 */
	pub fn start(&mut self)
	{
		for _elem in 0..5 
		{
			self.task[_elem].active = TaskState::WaitActivation;
			self.task[_elem].duration = self.plantconfig[_elem].duration;
		}
	}

	pub fn SetPlantConfig(&mut self, plant_index : usize,  config : WaterConfig)
	{
		if plant_index > max_number_of_plants
		{
			return;
		}

		self.plantconfig[plant_index] = config;
	}

	pub fn SetPlantConfig_all(&mut self, config : PlantData)
	{
		self.plantconfig = config;
	}

	pub fn cycling(&mut self, time_now : Time)
	{
		for _elem in 0..5 
		{
			match self.task[_elem].active
			{
				TaskState::Active => {
					if time_now >= self.task[_elem].end
					{
						self.relais_off(_elem);
						self.task[_elem].end = time!(0:00);
						self.task[_elem].active = TaskState::Off;
						return;
					}
					else
					{
						return;
					}
				}
				TaskState::WaitActivation => {
					self.relais_on(_elem);
					self.task[_elem].end = time_now + Duration::microseconds(self.task[_elem].duration.into());
					self.task[_elem].active = TaskState::Active;
					return;
				}
				TaskState::Off => {}, //do nothing
			}
		}
	}
	
	pub fn relais_on(&mut self, index : usize)
	{
		if index == 0 {
			self.relais1.set_high();
		} else if index == 1 {
			self.relais2.set_high();			
		} else if index == 2 {
			self.relais3.set_high();			
		} else if index == 3 {
			self.relais4.set_high();
		} else if index == 4 {
			self.relais5.set_high();
		} else if index == 5 {
			self.relais6.set_high();			
		} else {
			// error 
		}
	}
	
	pub fn relais_off(&mut self, index : usize)
	{
		if index == 0 {
			self.relais1.set_low();
		} else if index == 1 {
			self.relais2.set_low();			
		} else if index == 2 {
			self.relais3.set_low();			
		} else if index == 3 {
			self.relais4.set_low();
		} else if index == 4 {
			self.relais5.set_low();
		} else if index == 5 {
			self.relais6.set_low();			
		} else {
			// error 
		}	
	}
	
	/**
	 * @details the water needs be pured sensible or the apartment will become a pool 
	 * @todo 	switch to getTickcount or use actual time later 
	 */
	fn relaislogic(&mut self, index : usize)
	{
		let max_on_time = 6; // 2 seconds
		let max_off_time = 10; // 5 seconds 
		
		match self.relaisstate 
		{
			Relaisstate::Off => if self.count < max_off_time 
								{
									self.relais_off(index); // this isn't nessarry, but I don't have fait in the state logic
									self.count += 1;
								} 
								else 
								{
									self.relais_on(index);
									self.relaisstate = Relaisstate::On;
									self.count = 0;
								}
			Relaisstate::On => if self.count < max_on_time 
								{
									self.relais_on(index); // this isn't nessarry, but I don't have fait in the state logic
									self.count += 1;
								} 
								else 
								{
									self.relais_off(index);
									self.relaisstate = Relaisstate::Off;
									self.count = 0;
								}
		}
		
	}
	
	/**
	 * @param[sensor_voltage] 	Voltage of capatic earth moistre 
	 * @details 	on manual tests I gathered 2300mV for dry and 1300mV for fresh watered
	 * 				so 2100mV and 1500mV as borders are based on a whim
	 * @todo 		if the code has multible Pumps, relais, sensors, there must be a
	 * 			    check on how may relais are active. My supply is only able to power 
	 *   			1 or 2 pumps simustanually 
					additionaL: we must protect the Pumps. don't let them run for more than 
					5 minutes  
					the pump is rather strong (for its price lol) only do short intervals
	 */
	pub fn check_water_level(&mut self, sensor_voltage : [u16; 6])
	{
		//let dry_border = 2100;
		//let wet_border = 1500;
		let mut index = 0;
		
		while index < 6
		{
			if sensor_voltage[index] > self.plantconfig[index].levelhigh {
				self.relaislogic(index);
				rprintln!("Set Relais {}", index);
				return; 	// only one relais allowed
			}
			else if sensor_voltage[index] < self.plantconfig[index].levellow {
				self.relais_off(index);
				rprintln!("Clear Relais {}", index);					
			}
			index += 1;
		}
	}

	pub fn set_duration(&mut self, index: usize, duration : u16)
	{
		if index < 6 
		{ 
			self.plantconfig[index].duration = duration;
		}
	}
	  
	pub fn get_duration(&mut self, index: usize) -> u16 
	{
		if index < 6 
		{
			return self.plantconfig[index].duration;
		}
		else {
			return 0;
		}
	}

	
	pub fn relais_test(&mut self, index : usize)
	{
		if 0 == index 
		{
			self.relais_on(0);
			self.relais_off(1);
			self.relais_off(2);
			self.relais_off(3);
			self.relais_off(4);
			self.relais_off(5);
		} 
		else if index == 1 
		{				
			self.relais_off(0);
			self.relais_on(1);
			self.relais_off(2);
			self.relais_off(3);
			self.relais_off(4);
			self.relais_off(5);
		} 
		else if index == 2 
		{	
			self.relais_off(0);
			self.relais_off(1);
			self.relais_on(2);
			self.relais_off(3);
			self.relais_off(4);
			self.relais_off(5);	
		} 
		else if index == 3 
		{
			self.relais_off(0);
			self.relais_off(1);
			self.relais_off(2);
			self.relais_on(3);
			self.relais_off(4);
			self.relais_off(5);
		} 
		else if index == 4 
		{
			self.relais_off(0);
			self.relais_off(1);
			self.relais_off(2);
			self.relais_off(3);
			self.relais_on(4);
			self.relais_off(5);
		} 
		else if index == 5 
		{	
			self.relais_off(0);
			self.relais_off(1);
			self.relais_off(2);
			self.relais_off(3);
			self.relais_off(4);
			self.relais_on(5);	
		} 
		else 
		{
			self.relais_off(0);
			self.relais_off(1);
			self.relais_off(2);
			self.relais_off(3);
			self.relais_off(4);
			self.relais_off(5);
		}
	}
}
