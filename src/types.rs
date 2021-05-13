use stm32f1xx_hal::gpio::*;

pub type BeeperPin = gpioa::PA2<Output<PushPull>>;
