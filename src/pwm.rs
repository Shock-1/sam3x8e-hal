extern crate embedded_hal as hal;

use crate::pmc::Clocks;
use sam3x8e::Peripherals;

struct PWM {
    peripherals: Peripherals,
    clocks: Clocks,
}

impl PWM {
    fn new(peripherals: Peripherals, clocks: Clocks) -> Self {
        PWM {
            peripherals: peripherals,
            clocks: clocks,
        }
    }
}

#[derive(PartialEq)]
pub enum Channel {
    CHID0 = 0,
    CHID1 = 1,
    CHID2 = 2,
    CHID3 = 3,
    CHID4 = 4,
    CHID5 = 5,
    CHID6 = 6,
    CHID7 = 7,
}

// This implementation strives to do something useful over being perfect, as
// the "unproven" hal::Pwm interface can't express the set of things
// available on SAM3X

// Prescaler is set to be the Master Clock (MCK) directly
const WPKEY: u32 = 0x50574D;
const CPRE: u8 = 0b0000; // Master Clock directly; i.e. No Prescaler
const PRESCALER: f32 = 1.0;

impl hal::Pwm for PWM {
    type Channel = Channel;
    type Time = f32; // Seconds
    type Duty = f32; // 0.0 ... 1.0

    fn enable(&mut self, channel: Self::Channel) {
        self.peripherals.PWM.wpcr.write_with_zero(|w| unsafe {
            w.wpkey().bits(WPKEY).wpcmd().bits(0).wprg1().set_bit()
        }); 

        let pwm_sr = self.peripherals.PWM.sr.read();
        let channel_enabled = match channel {
            Channel::CHID0 => pwm_sr.chid0().bit_is_set(),
            Channel::CHID1 => pwm_sr.chid1().bit_is_set(),
            Channel::CHID2 => pwm_sr.chid2().bit_is_set(),
            Channel::CHID3 => pwm_sr.chid3().bit_is_set(),
            Channel::CHID4 => pwm_sr.chid4().bit_is_set(),
            Channel::CHID5 => pwm_sr.chid5().bit_is_set(),
            Channel::CHID6 => pwm_sr.chid6().bit_is_set(),
            Channel::CHID7 => pwm_sr.chid7().bit_is_set(),
        };
        if channel_enabled {
            self.peripherals.PWM.dis.write_with_zero(|w| match channel {
                Channel::CHID0 => w.chid0().set_bit(),
                Channel::CHID1 => w.chid1().set_bit(),
                Channel::CHID2 => w.chid2().set_bit(),
                Channel::CHID3 => w.chid3().set_bit(),
                Channel::CHID4 => w.chid4().set_bit(),
                Channel::CHID5 => w.chid5().set_bit(),
                Channel::CHID6 => w.chid6().set_bit(),
                Channel::CHID7 => w.chid7().set_bit(),
            });
        }

        // CALG is cleared, all PWM is left-aligned
        // CPOL is set, output waveform starts high
        match channel {
            Channel::CHID0 => self.peripherals.PWM.cmr0.write_with_zero(|w| unsafe { w.cpre().bits(CPRE).cpol().set_bit().calg().clear_bit() }),
            Channel::CHID1 => self.peripherals.PWM.cmr1.write_with_zero(|w| unsafe { w.cpre().bits(CPRE).cpol().set_bit().calg().clear_bit() }),
            Channel::CHID2 => self.peripherals.PWM.cmr2.write_with_zero(|w| unsafe { w.cpre().bits(CPRE).cpol().set_bit().calg().clear_bit() }),
            Channel::CHID3 => self.peripherals.PWM.cmr3.write_with_zero(|w| unsafe { w.cpre().bits(CPRE).cpol().set_bit().calg().clear_bit() }),
            Channel::CHID4 => self.peripherals.PWM.cmr4.write_with_zero(|w| unsafe { w.cpre().bits(CPRE).cpol().set_bit().calg().clear_bit() }),
            Channel::CHID5 => self.peripherals.PWM.cmr5.write_with_zero(|w| unsafe { w.cpre().bits(CPRE).cpol().set_bit().calg().clear_bit() }),
            Channel::CHID6 => self.peripherals.PWM.cmr6.write_with_zero(|w| unsafe { w.cpre().bits(CPRE).cpol().set_bit().calg().clear_bit() }),
            Channel::CHID7 => self.peripherals.PWM.cmr7.write_with_zero(|w| unsafe { w.cpre().bits(CPRE).cpol().set_bit().calg().clear_bit() }),
        }

        self.peripherals.PWM.ena.write_with_zero(|w| match channel {
            Channel::CHID0 => w.chid0().set_bit(),
            Channel::CHID1 => w.chid1().set_bit(),
            Channel::CHID2 => w.chid2().set_bit(),
            Channel::CHID3 => w.chid3().set_bit(),
            Channel::CHID4 => w.chid4().set_bit(),
            Channel::CHID5 => w.chid5().set_bit(),
            Channel::CHID6 => w.chid6().set_bit(),
            Channel::CHID7 => w.chid7().set_bit(),
        });
    }

    fn disable(&mut self, channel: Self::Channel) {
        self.peripherals.PWM.wpcr.write_with_zero(|w| unsafe {
            w.wpkey().bits(WPKEY).wpcmd().bits(0).wprg1().set_bit()
        });
        self.peripherals.PWM.dis.write_with_zero(|w| match channel {
            Channel::CHID0 => w.chid0().set_bit(),
            Channel::CHID1 => w.chid1().set_bit(),
            Channel::CHID2 => w.chid2().set_bit(),
            Channel::CHID3 => w.chid3().set_bit(),
            Channel::CHID4 => w.chid4().set_bit(),
            Channel::CHID5 => w.chid5().set_bit(),
            Channel::CHID6 => w.chid6().set_bit(),
            Channel::CHID7 => w.chid7().set_bit(),
        })
    }

    fn get_period(&self) -> Self::Time {
        // This is a bit ambiguous on this platform, since each of the 8
        // channels could have their own periods.
        // Do something here and find the first enabled channel and return that
        // period.
        let sr = self.peripherals.PWM.sr.read();
        let master_clock_frequency= self.clocks.master_clock_freq().0 as f32;

        let cprd = 
            if sr.chid0().bit_is_set() { self.peripherals.PWM.cprd0.read().cprd().bits() }
            else if sr.chid1().bit_is_set() { self.peripherals.PWM.cprd1.read().cprd().bits() }
            else if sr.chid2().bit_is_set() { self.peripherals.PWM.cprd2.read().cprd().bits() }
            else if sr.chid3().bit_is_set() { self.peripherals.PWM.cprd3.read().cprd().bits() }
            else if sr.chid4().bit_is_set() { self.peripherals.PWM.cprd4.read().cprd().bits() }
            else if sr.chid5().bit_is_set() { self.peripherals.PWM.cprd5.read().cprd().bits() }
            else if sr.chid6().bit_is_set() { self.peripherals.PWM.cprd6.read().cprd().bits() }
            else if sr.chid7().bit_is_set() { self.peripherals.PWM.cprd7.read().cprd().bits() }
            else { 0 }
        ;
        if cprd == 0 {
            0.0
        } else {
            (PRESCALER * cprd as f32) / master_clock_frequency
        }
    }

    fn get_duty(&self, channel: Self::Channel) -> Self::Duty {
        match channel {
            Channel::CHID0 => {self.peripherals.PWM.cdty0.read().cdty().bits() as f32 / self.peripherals.PWM.cprd0.read().cprd().bits() as f32},
            Channel::CHID1 => {self.peripherals.PWM.cdty1.read().cdty().bits() as f32 / self.peripherals.PWM.cprd1.read().cprd().bits() as f32},
            Channel::CHID2 => {self.peripherals.PWM.cdty2.read().cdty().bits() as f32 / self.peripherals.PWM.cprd2.read().cprd().bits() as f32},
            Channel::CHID3 => {self.peripherals.PWM.cdty3.read().cdty().bits() as f32 / self.peripherals.PWM.cprd3.read().cprd().bits() as f32},
            Channel::CHID4 => {self.peripherals.PWM.cdty4.read().cdty().bits() as f32 / self.peripherals.PWM.cprd4.read().cprd().bits() as f32},
            Channel::CHID5 => {self.peripherals.PWM.cdty5.read().cdty().bits() as f32 / self.peripherals.PWM.cprd5.read().cprd().bits() as f32},
            Channel::CHID6 => {self.peripherals.PWM.cdty6.read().cdty().bits() as f32 / self.peripherals.PWM.cprd6.read().cprd().bits() as f32},
            Channel::CHID7 => {self.peripherals.PWM.cdty7.read().cdty().bits() as f32 / self.peripherals.PWM.cprd7.read().cprd().bits() as f32},
        }
    }

    fn get_max_duty(&self) -> Self::Duty {
        1.0
    }

    fn set_duty(&mut self, channel: Self::Channel, duty: Self::Duty) {
        // duty_f = duty_u / period_u
        // duty_f * period_u = duty_u
        let cprd = match channel {
            Channel::CHID0 => self.peripherals.PWM.cprd0.read().cprd().bits() as f32,
            Channel::CHID1 => self.peripherals.PWM.cprd1.read().cprd().bits() as f32,
            Channel::CHID2 => self.peripherals.PWM.cprd2.read().cprd().bits() as f32,
            Channel::CHID3 => self.peripherals.PWM.cprd3.read().cprd().bits() as f32,
            Channel::CHID4 => self.peripherals.PWM.cprd4.read().cprd().bits() as f32,
            Channel::CHID5 => self.peripherals.PWM.cprd5.read().cprd().bits() as f32,
            Channel::CHID6 => self.peripherals.PWM.cprd6.read().cprd().bits() as f32,
            Channel::CHID7 => self.peripherals.PWM.cprd7.read().cprd().bits() as f32,
        };
        let duty_u = (duty * cprd) as u32;
        match channel {
            Channel::CHID0 => self.peripherals.PWM.cdty0.write_with_zero(|w| unsafe { w.cdty().bits(duty_u)}),
            Channel::CHID1 => self.peripherals.PWM.cdty1.write_with_zero(|w| unsafe { w.cdty().bits(duty_u)}),
            Channel::CHID2 => self.peripherals.PWM.cdty2.write_with_zero(|w| unsafe { w.cdty().bits(duty_u)}),
            Channel::CHID3 => self.peripherals.PWM.cdty3.write_with_zero(|w| unsafe { w.cdty().bits(duty_u)}),
            Channel::CHID4 => self.peripherals.PWM.cdty4.write_with_zero(|w| unsafe { w.cdty().bits(duty_u)}),
            Channel::CHID5 => self.peripherals.PWM.cdty5.write_with_zero(|w| unsafe { w.cdty().bits(duty_u)}),
            Channel::CHID6 => self.peripherals.PWM.cdty6.write_with_zero(|w| unsafe { w.cdty().bits(duty_u)}),
            Channel::CHID7 => self.peripherals.PWM.cdty7.write_with_zero(|w| unsafe { w.cdty().bits(duty_u)}),
        }
    }

    fn set_period<P>(&mut self, period: P)
    where
            P: Into<Self::Time> {
        let cprd = ((period.into() * self.clocks.master_clock_freq().0 as f32) / PRESCALER) as u32;
        self.peripherals.PWM.wpcr.write_with_zero(|w| unsafe { w.wpkey().bits(WPKEY).wpcmd().bits(0).wprg3().set_bit() });
        self.peripherals.PWM.cprd0.write_with_zero(|w| unsafe { w.cprd().bits(cprd) });
        self.peripherals.PWM.cprd1.write_with_zero(|w| unsafe { w.cprd().bits(cprd) });
        self.peripherals.PWM.cprd2.write_with_zero(|w| unsafe { w.cprd().bits(cprd) });
        self.peripherals.PWM.cprd3.write_with_zero(|w| unsafe { w.cprd().bits(cprd) });
        self.peripherals.PWM.cprd4.write_with_zero(|w| unsafe { w.cprd().bits(cprd) });
        self.peripherals.PWM.cprd5.write_with_zero(|w| unsafe { w.cprd().bits(cprd) });
        self.peripherals.PWM.cprd6.write_with_zero(|w| unsafe { w.cprd().bits(cprd) });
        self.peripherals.PWM.cprd7.write_with_zero(|w| unsafe { w.cprd().bits(cprd) });
    }
}

