#![no_std]

//pub use hal::stm32 as pac;
pub use stm32f4xx_hal as hal;

mod led;
pub use led::Led;

mod button;
pub use button::Button;

mod watering_logic;
pub use watering_logic::Watering;

mod serial_interface;
pub use serial_interface::{SerialInterface, SerialCommand};