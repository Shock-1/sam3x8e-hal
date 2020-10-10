#![no_std]

extern crate embedded_hal as hal;

pub mod delay;
pub mod gpio;
pub mod pmc;
pub mod time;
pub mod pwm;

#[cfg(test)]
mod tests {

    #[test]
    fn it_works() {}
}
