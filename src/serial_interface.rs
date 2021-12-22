/** 
 *  This modul should print the current parameter and be an interface
 *  for changeing them I first thought of an interface. I I think key
 *  bindings would be easier.
 * milestones:
 * * Print something
 * * clear screen 
 * + Print time
 * * get serial input 
 * + have time changeable
 * 
 * Readable via: cu -s 115200 -l /dev/tty.usbmodem14103  -> but not suited for typing charracters
 * so something like putty is better
 */

pub enum SerialCommand
{
    Empty,
    PrintTime, 
    PrintNext,
    PrintUsage,
    ActivatePump(u8),
    SetDate(NaiveDate),
    SetTime(NaiveTime),
    SetNext(NaiveDate),
    SetPlantConfig(u8, u32),
    SetPlantWateringDuration(u32, u32),
}

use stm32f4xx_hal::{
        prelude::*,
        serial::{Tx, Rx},
    };

use stm32f4xx_hal::stm32::{USART2};
use stm32f4xx_hal::nb::block;

use arrayvec::ArrayString; // 0.4.10
use ascii::{AsciiChar};
use core::fmt::Write;

use rtcc::{NaiveDate, NaiveDateTime, NaiveTime};
use rtt_target::{rprintln};
  
pub struct SerialInterface {
    tx : Tx<USART2>,
    rx : Rx<USART2>,
    readbuffer : ArrayString::<20>,
    commandcomplete : u8,
}

impl SerialInterface {
    
    
    pub fn new(tx : Tx<USART2>, rx : Rx<USART2>) -> Self 
    {
        Self {tx, rx, readbuffer : ArrayString::<20>::new(), commandcomplete : 0}
    }

    pub fn init(&mut self) 
    {
    	self.rx.listen();
    }

    /**
     * found on stackoverflow
     * this only works on real terminal programs, like putty or cu
     */
    fn clearscreen(&mut self){
        block!(self.tx.write(27)).ok();             // ESC
        for element in b"[2J" 
        {                     // ESC + [2J clear screen
            block!(self.tx.write(*element)).ok();   
        }
        block!(self.tx.write(27)).ok();
        for element in b"[H" 
        {
            block!(self.tx.write(*element)).ok();    // ESC + [H  goes to home location
        }
    }

    // this need to extend to a command interface. React on newline, if now collect for command
    // change Print behavior. 
    // old: clear and write 
    // new: collect imput and write only on command
    fn listenserialline (&mut self)
    {
    	if self.rx.is_rx_not_empty() {
    		//rprintln!("Some Rx");
    		match self.rx.read() {
    			Ok(data) => { 
                    // let received = block!(self.rx.read()).unwrap();
                    // let character = AsciiChar::from(received).unwrap();
                    let character = AsciiChar::from(data).unwrap();
                    if character == AsciiChar::CarriageReturn
                    {
                        // command complete -> 
                        self.commandcomplete = 1;
                        block!(self.tx.write(b'\r')).ok();
                        block!(self.tx.write(b'\n')).ok();
                    }
                    else if character == AsciiChar::DEL 
                    {
                        self.readbuffer.pop();
                    }
                    else 
                    {
    				    self.readbuffer.try_push(character.as_char());
                        block!(self.tx.write(character.as_byte())).ok();
                    }
    			//readbuffer[bufferindex]; bufferindex += 1;
    			}, 
    			Err(error) => {
    				rprintln!("Serial read error {:?}",error)
    			},
    		}
    	}
    }
    
    pub fn check_for_command(&mut self) -> SerialCommand
    {
        let mut command : SerialCommand = SerialCommand::Empty;        
        if self.commandcomplete == 1
        {
            //let commandstring = self.readbuffer.make_ascii_lowercase();
            // let commandstring = self.readbuffer.matches("time").collect();
            // if !commandstring.empty()
            // {
            //     command = SerialCommand::PrintTime;
            // }
            if self.readbuffer.contains("time")
            {
                command = SerialCommand::PrintTime;
            }
            else if self.readbuffer.contains("next")
            {
                command = SerialCommand::PrintNext;
            }
            else if self.readbuffer.contains("sett")
            {
                self.readbuffer.remove(0); // I should write myself a parser 
                self.readbuffer.remove(0);
                self.readbuffer.remove(0);
                self.readbuffer.remove(0);
                self.readbuffer.remove(0);
                match NaiveTime::parse_from_str(self.readbuffer.as_str(), "%H:%M:%S").ok()
                {
                    None => {},
                    Some(time) => command = SerialCommand::SetTime(time),
                }
            }
            else if self.readbuffer.contains("setd")
            {
                self.readbuffer.remove(0);  
                self.readbuffer.remove(0);
                self.readbuffer.remove(0);
                self.readbuffer.remove(0);
                self.readbuffer.remove(0);
                match NaiveDate::parse_from_str(self.readbuffer.as_str(), "%d.%m.%Y").ok()
                {
                    None => {},
                    Some(date) => command = SerialCommand::SetDate(date),
                }
            }
            else if self.readbuffer.contains("setnxt")
            {
                self.readbuffer.remove(0);  
                self.readbuffer.remove(0);
                self.readbuffer.remove(0);
                self.readbuffer.remove(0);
                self.readbuffer.remove(0);
                self.readbuffer.remove(0);
                self.readbuffer.remove(0);
                match NaiveDate::parse_from_str(self.readbuffer.as_str(), "%d.%m.%Y").ok()
                {
                    None => {},
                    Some(date) => command = SerialCommand::SetNext(date),
                }
            }
            else if self.readbuffer.contains("activ")
            {
                self.readbuffer.remove(0);  
                self.readbuffer.remove(0);
                self.readbuffer.remove(0);
                self.readbuffer.remove(0);
                self.readbuffer.remove(0);
                self.readbuffer.remove(0);
                match AsciiChar::from(self.readbuffer.remove(0)).ok()
                {
                    Some(AsciiChar::_1) => command = SerialCommand::ActivatePump(0),  // we only have 6 Pumps 
                    Some(AsciiChar::_2) => command = SerialCommand::ActivatePump(1),
                    Some(AsciiChar::_3) => command = SerialCommand::ActivatePump(2),
                    Some(AsciiChar::_4) => command = SerialCommand::ActivatePump(3),
                    Some(AsciiChar::_5) => command = SerialCommand::ActivatePump(4),
                    Some(AsciiChar::_6) => command = SerialCommand::ActivatePump(5),
                    None => {}, // todo print usage on error 
                    _ => {},
                }
            }
            else if self.readbuffer.contains("dur")
            {   
                let mut position = 0;
                let mut index = 9;
                let mut duration = 0;
                // dur 1 5000 => first pump 
                // splitting is not possible on no_std -> we do it ourself 
                for element in self.readbuffer.chars()
                {
                    if element.is_numeric()
                    {
                        if position == 0
                        {
                            match element.to_digit(10)
                            {
                                Some(number) => index = number,
                                _ => break,
                            }
                            position += 1;
                        }
                        else if position == 1
                        {
                            duration = duration *10;
                            match element.to_digit(10)
                            {
                                Some(number) => duration += number,
                                _ => break,
                            }
                            
                        }
                    }
                }
                
                command = SerialCommand::SetPlantWateringDuration(index, duration);
            }
            else 
            {
                command = SerialCommand::PrintUsage;
            }

            self.readbuffer.clear();
            self.commandcomplete = 0;
        }

        return command;
    }

    // todo use private constants for print and check?
    pub fn printcommands(&mut self)
    {
        let mut buf = ArrayString::<104>::new();
        write!(&mut buf, "supported commands:\r\ntime - prints current time\r\nnext\r\nsett\r\nsetd\r\nsetnxt\r\ndur\r\nactive\r\n").expect("Can't write");
        for element in buf.bytes() 
        {
            block!(self.tx.write(element)).ok();    // ESC + [H  goes to home location
        }
    }
    
    pub fn printmessage(&mut self, msg : &str) 
    {
        let mut buf = ArrayString::<20>::new();
        write!(&mut buf, "{:}", msg).expect("Can't write");
        for element in buf.bytes() 
        {
            block!(self.tx.write(element)).ok();    // ESC + [H  goes to home location
        }
    }
    
    fn print_something(&mut self)
    {
        let mut buf = ArrayString::<20>::new();
        
        write!(&mut buf, "Here is something to Print").expect("Can't write");
		
		for element in buf.bytes() {
            block!(self.tx.write(element)).ok();    // ESC + [H  goes to home location
        }
    }	

    pub fn print_time(&mut self, time : NaiveDateTime)
    {
        let mut buf = ArrayString::<30>::new();
        write!(&mut buf, "Time {:?}\r\n", time).expect("Can't write");
        
		for element in buf.bytes() {
            block!(self.tx.write(element)).ok();    // ESC + [H  goes to home location
        }
    }	
	/**
     * intended to be called every programm cycle.
     * - clear field and reprint new time.
     * - print curser position
     * - print status?
     */
	pub fn printcycle(&mut self){
        //self.clearscreen();
        self.listenserialline();
        //self.print_something();
    } 
}
