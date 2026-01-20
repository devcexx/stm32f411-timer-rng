#![no_std]

use stm32f4_staging::stm32f411::{RCC, TIM5};

pub fn gen_random_bytes(rcc: &mut RCC, _tim5: &mut TIM5, buf: &mut [u8]) {
    let rng = TimerRng::new(rcc, unsafe {
        // SAFETY: Since we have a mut reference to TIM5, we can assume exclusivity over it.
        TIM5::steal()
    });

    rng.next_bytes(buf);
    rng.restore_and_release(rcc);
}

pub fn gen_random_byte_array<const N: usize>(rcc: &mut RCC, tim5: &mut TIM5) -> [u8; N] {
    let mut buf = [0u8; N];
    gen_random_bytes(rcc, tim5, &mut buf);
    buf
}

pub fn gen_random_u64(rcc: &mut RCC, tim5: &mut TIM5) -> u64 {
    u64::from_be_bytes(gen_random_byte_array(rcc, tim5))
}

pub struct TimerRng {
    tim5: TIM5,
    tim_was_powered: bool,
    lsi_was_on: bool,
    old_it4_rmp: u8
}

impl TimerRng {
    fn tim_set_enabled(tim: &mut TIM5, enabled: bool) {
        tim.cr1().modify(|_, w| {
            w.cen().bit(enabled)
        });
    }

    fn tim_set_powered(rcc: &mut RCC, powered: bool) -> bool {
        let mut tim_was_powered: bool = false;
        rcc.apb1enr().modify(|r, w| {
            tim_was_powered = r.tim5en().bit_is_set();
            w.tim5en().bit(powered)
        });

        tim_was_powered
    }


    fn lsi_set_on(rcc: &mut RCC, on: bool) -> bool {
        let mut lsi_was_on: bool = false;
        rcc.csr().modify(|r, w| {
            lsi_was_on = r.lsion().bit_is_set();
            w.lsion().bit(on)
        });

        if on {
             while rcc.csr().read().lsirdy().bit_is_clear() {}
        }

        lsi_was_on
    }

    unsafe fn set_it4_rmp(tim: &mut TIM5, value: u8) -> u8 {
        let mut old_it4_rmp = 0u8;
        tim.or().modify(|r, w| unsafe {
            old_it4_rmp = r.it4_rmp().bits();
            w.it4_rmp().bits(value)
        });
        old_it4_rmp
    }

    pub fn new(rcc: &mut RCC, mut tim5: TIM5) -> Self {
        let tim_was_powered: bool = Self::tim_set_powered(rcc, true);
        let lsi_was_on: bool = Self::lsi_set_on(rcc, true);


        Self::tim_set_enabled(&mut tim5, false);

        // TIM5 auto reload value
        tim5.arr().write(|w| w.set(u32::MAX));

        // Use TI4 as source for TIM5_CH4
        tim5.ccmr2_input().modify(|_, w| {
            unsafe {
                // Cannot use cc4s().ti4() because in stm32f4-staging 0.16 (The
                // version used in stm32f4xx_hal), this is broken, because it
                // swaps ti4() with ti3(). Fixed in 0.20. Just writing the raw
                // value directly.
                w.cc4s().bits(0b01)
            }
        });

        let old_it4_rmp = unsafe {
             // Connect TIM5_CH4 to LSI, instead of to the GPIO as normally.
            Self::set_it4_rmp(&mut tim5, 0b01)
        };



        tim5.ccer().modify(|_, w| {
            w.cc4np().clear_bit() // trigger on edge raise
            .cc4p().clear_bit() // trigger on edge raise
            .cc4e().set_bit() // Enable TIM5_CH4
        });


        tim5.cr1().modify(|_, w| {
            w.dir().up() // Upcount
        });
        Self::tim_set_enabled(&mut tim5, true);

        Self {
            tim5,
            lsi_was_on,
            tim_was_powered,
            old_it4_rmp
        }
    }

    pub fn next_byte(&self) -> u8 {
        while self.tim5.sr().read().cc4if().bit_is_clear() {}
        (self.tim5.ccr4().read().bits() & 0xff) as u8
    }

    pub fn next_bytes(&self, buf: &mut [u8]) {
        buf.fill_with(|| {
            self.next_byte()
        });
    }

    pub fn release(mut self) -> TIM5 {
        Self::tim_set_enabled(&mut self.tim5, false);
        unsafe {
            Self::set_it4_rmp(&mut self.tim5, self.old_it4_rmp);
        }
        self.tim5
    }

    pub fn restore_and_release(self, rcc: &mut RCC) -> TIM5 {
        Self::tim_set_powered(rcc, self.tim_was_powered);
        Self::lsi_set_on(rcc, self.lsi_was_on);
        let tim = self.release();
        tim
    }
}

#[cfg(feature = "rand")]
mod rand;
#[cfg(feature = "rand")]
pub use rand::*;
