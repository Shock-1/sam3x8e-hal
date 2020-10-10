//! Power Mode Controller (Manages clocks)

use core::cmp;

use crate::time::{Hertz, U32Ext};
use sam3x8e::{pmc, PMC};

/// Extension trait that constraints the 'pmc' peripheral
pub trait PmcExt {
    /// Constraints the pmc peripheral so it plays nicely with other abstractions
    fn constraint(self) -> Pmc;
}

/// Constrained pmc peripheral
pub struct Pmc {
    /// Peripheral clocks controlling pins from 8 to 31
    pub pclk0: Pclk0,
    /// Peripheral clocks controlling pins from 32 to 44
    pub pclk1: Pclk1,
    /// Clock configuration
    pub cfgr: CFGR,
}

impl PmcExt for Pmc {
    fn constraint(self) -> Pmc {
        Pmc {
            pclk0: Pclk0 { _0: () },
            pclk1: Pclk1 { _0: () },
            cfgr: CFGR {
                master_clock: None,
                clock_source: ClockSource::SlowClock,
            },
        }
    }
}

/// Peripheral clocks controlling pins from 8 to 31
pub struct Pclk0 {
    _0: (),
}

impl Pclk0 {
    pub(crate) fn er(&mut self) -> &pmc::PMC_PCER0 {
        // NOTE(unsafe) this proxy grants exclusive access to this register
        unsafe { &(*PMC::ptr()).pmc_pcer0 }
    }

    pub(crate) fn dr(&mut self) -> &pmc::PMC_PCDR0 {
        // NOTE(unsafe) this proxy grants exclusive access to this register
        unsafe { &(*PMC::ptr()).pmc_pcdr0 }
    }

    pub(crate) fn sr(&mut self) -> &pmc::PMC_PCSR0 {
        // NOTE(unsafe) this proxy grants exclusive access to this register
        unsafe { &(*PMC::ptr()).pmc_pcsr0 }
    }
}

/// Peripheral clocks controlling pins from 32 to 44
pub struct Pclk1 {
    _0: (),
}

impl Pclk1 {
    pub(crate) fn er(&mut self) -> &pmc::PMC_PCER1 {
        // NOTE(unsafe) this proxy grants exclusive access to this register
        unsafe { &(*PMC::ptr()).pmc_pcer1 }
    }

    pub(crate) fn dr(&mut self) -> &pmc::PMC_PCDR1 {
        // NOTE(unsafe) this proxy grants exclusive access to this register
        unsafe { &(*PMC::ptr()).pmc_pcdr1 }
    }

    pub(crate) fn sr(&mut self) -> &pmc::PMC_PCSR1 {
        // NOTE(unsafe) this proxy grants exclusive access to this register
        unsafe { &(*PMC::ptr()).pmc_pcsr1 }
    }
}

const SLOW_CLOCK_FREQ: u32 = 32_768; //Hz

/// Possible sources for Master clock
#[derive(Copy, Clone)]
pub enum ClockSource {
    MainClock,
    SlowClock,
    PllClock,
    //TODO: Support UPLLCK
}

/// Clock configuration
pub struct CFGR {
    /// Master Clock frequency
    master_clock: Option<u32>,
    //TODO: Add support for programmable clocks
    /// Master Clock's source clock
    clock_source: ClockSource,
}

impl CFGR {
    pub fn new() -> CFGR {
        return CFGR{master_clock: None, clock_source: ClockSource::SlowClock}
    }
    ///Assign desired Master clock frequency
    pub fn master_clock(mut self, freq: impl Into<Hertz>) -> Self {
        self.master_clock = Some(freq.into().0);
        self
    }
    ///Change clock source
    pub fn clock_source(mut self, src: ClockSource) -> Self {
        self.clock_source = src;
        self
    }

    ///Freezes the clock frequencies making it effective
    pub fn freeze(self) -> Clocks {
        use sam3x8e::generic::Variant::Val;

        let pmc = unsafe { &(*PMC::ptr()) };
        let mut mck = self.master_clock.unwrap_or(SLOW_CLOCK_FREQ);
        let mut pres = 1u16;
        let main_clock_freq = match pmc.ckgr_mor.read().moscrcf().variant() {
            Val(pmc::ckgr_mor::MOSCRCF_A::_4_MHZ) => 4_000_000, //Hz
            Val(pmc::ckgr_mor::MOSCRCF_A::_8_MHZ) => 8_000_000, //Hz
            Val(pmc::ckgr_mor::MOSCRCF_A::_12_MHZ) => 12_000_000, //Hz
            _ => unreachable!(),
        };

        match self.clock_source {
            ClockSource::PllClock => {
                let pllmul: u16 =
                    2 * (self.master_clock.unwrap_or(main_clock_freq) / main_clock_freq) as u16;
                let pllmul = cmp::min(cmp::max(pllmul, 2), 2048);

                //Actually safe as max value is guaranteed to be 2048
                pmc.ckgr_pllar
                    .write(|w| unsafe { w.diva().bits(2).mula().bits(pllmul - 1) });
                while pmc.pmc_sr.read().locka().bit_is_clear() {}

                pmc.pmc_mckr.write(|w| {
                    //TODO: Think of something that utilizes the pre-scaler
                    w.pres().clk_1();
                    while pmc.pmc_sr.read().mckrdy().bit_is_clear() {}
                    w.css().plla_clk();
                    while pmc.pmc_sr.read().mckrdy().bit_is_clear() {}
                    w
                });
                mck = main_clock_freq * u32::from(pllmul)
            }
            ClockSource::SlowClock => {
                let div = SLOW_CLOCK_FREQ / mck;

                assert!(mck <= SLOW_CLOCK_FREQ);

                let pres_bits = match div {
                    0 => unreachable!(),
                    1 => 0,
                    2 => 1,
                    3 => 7,
                    4 => 2,
                    5..=8 => 3,
                    9..=16 => 4,
                    17..=32 => 5,
                    _ => 6,
                };

                pmc.pmc_mckr.write(|w| {
                    w.css().slow_clk();
                    while pmc.pmc_sr.read().mckrdy().bit_is_clear() {}
                    w.pres().bits(pres_bits);
                    while pmc.pmc_sr.read().mckrdy().bit_is_clear() {}
                    w
                });
                pres = if div != 3 {
                    2u16.pow(pres_bits.into())
                } else {
                    3
                };
                mck /= u32::from(pres)
            }
            ClockSource::MainClock => {
                let div = main_clock_freq / mck;

                assert!(mck <= main_clock_freq);

                let pres_bits = match div {
                    0 => unreachable!(),
                    1 => 0,
                    2 => 1,
                    3 => 7,
                    4 => 2,
                    5..=8 => 3,
                    9..=16 => 4,
                    17..=32 => 5,
                    _ => 6,
                };

                pmc.pmc_mckr.write(|w| {
                    w.css().main_clk();
                    while pmc.pmc_sr.read().mckrdy().bit_is_clear() {}
                    w.pres().bits(pres_bits);
                    while pmc.pmc_sr.read().mckrdy().bit_is_clear() {}
                    w
                });
                pres = if div != 3 {
                    2u16.pow(pres_bits.into())
                } else {
                    3
                };
                mck /= u32::from(pres);
            }
        };
        Clocks {
            clock_source: self.clock_source,
            slck: SLOW_CLOCK_FREQ.hz(),
            main_clock_freq: main_clock_freq.hz(),
            pllack: (mck / main_clock_freq).hz(),
            master_clock_freq: mck.hz(),
            pres,
        }
    }
}

///Frozen clock frequencies
///
/// Existence of this value indicates that the clock configuration cannot be changed
#[derive(Copy, Clone)]
pub struct Clocks {
    clock_source: ClockSource,
    slck: Hertz,
    main_clock_freq: Hertz,
    pllack: Hertz,
    master_clock_freq: Hertz,
    pres: u16,
}

impl Clocks {
    /// Returns the clock source of main clock
    pub fn clock_source(&self) -> ClockSource {
        self.clock_source
    }
    /// Returns the frequency of slow clock
    pub fn slck(&self) -> Hertz {
        self.slck
    }
    /// returns the frequency of main clock
    pub fn main_clock_freq(&self) -> Hertz {
        self.main_clock_freq
    }

    /// Returns the frequency of PLLA clock
    pub fn pllack(&self) -> Hertz {
        self.pllack
    }

    /// Returns the frequency of Master Clock
    pub fn master_clock_freq(&self) -> Hertz {
        self.master_clock_freq
    }
    pub fn mck(&self) -> Hertz {
        self.master_clock_freq
    }

    /// Returns the value of prescaler in Master Clock controller
    pub fn pres(&self) -> u16 {
        self.pres
    }
}
