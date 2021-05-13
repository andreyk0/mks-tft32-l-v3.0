use crate::consts::*;
use cortex_m::asm;
use embedded_hal::blocking::delay::*;

const CYCLES_PER_MILLIS: u32 = SYS_FREQ.0 / 1_000;
const CYCLES_PER_MICROS: u32 = SYS_FREQ.0 / 1_000_000;

pub struct AsmDelay;

impl DelayMs<u16> for AsmDelay {
    fn delay_ms(&mut self, ms: u16) {
        asm::delay(CYCLES_PER_MILLIS * (ms as u32));
    }
}

impl DelayUs<u16> for AsmDelay {
    fn delay_us(&mut self, us: u16) {
        asm::delay(CYCLES_PER_MICROS * (us as u32));
    }
}
