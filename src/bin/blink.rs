#![cfg_attr(not(doc), no_main)]
#![no_std]

use panic_halt as _;

use core::convert::TryFrom;

use cortex_m::asm;
//use cortex_m_semihosting::hprintln;

use stm32f1xx_hal::prelude::*;

use rtic::cyccnt::Duration;

use stm32_rust_rtic_blink::{consts::*, delay::*, lcd::*, types::*};

use embedded_graphics::{
    egcircle, egrectangle,
    fonts::{Font6x8, Text},
    pixelcolor::Rgb565,
    prelude::*,
    primitive_style,
    style::TextStyleBuilder,
};

#[rtic::app(device = stm32f1xx_hal::stm32,
            peripherals = true,
            monotonic = rtic::cyccnt::CYCCNT)]
const APP: () = {
    struct Resources {
        beeper: BeeperPin,
        lcd: Lcd<AsmDelay>,
        cnt: u32,
    }

    #[init(schedule = [blink])]
    fn init(cx: init::Context) -> init::LateResources {
        let mut core: rtic::Peripherals = cx.core;
        let device = cx.device;
        let mut flash = device.FLASH.constrain();
        let mut rcc = device.RCC.constrain();

        let _clocks = rcc
            .cfgr
            .use_hse(8.mhz()) // TODO: should be 25Mhz!
            .sysclk(SYS_FREQ)
            .pclk1(36.mhz())
            .freeze(&mut flash.acr);

        //assert!(clocks.usbclk_valid());

        let mut gpioa = device.GPIOA.split(&mut rcc.apb2);
        let mut gpiob = device.GPIOB.split(&mut rcc.apb2);
        let mut gpioc = device.GPIOC.split(&mut rcc.apb2);
        let mut gpiod = device.GPIOD.split(&mut rcc.apb2);

        let beeper = gpioa.pa2.into_push_pull_output(&mut gpioa.crl);

        let lcd = Lcd::new(
            AsmDelay,
            device.GPIOE,
            &mut rcc.apb2,
            gpiod.pd14.into_push_pull_output(&mut gpiod.crh),
            gpioc.pc8.into_push_pull_output(&mut gpioc.crh),
            gpiod.pd13.into_push_pull_output(&mut gpiod.crh),
            gpiob.pb14.into_push_pull_output(&mut gpiob.crh),
            gpiod.pd15.into_push_pull_output(&mut gpiod.crh),
        )
        .unwrap();

        // Initialize (enable) the monotonic timer (CYCCNT)
        core.DCB.enable_trace();
        // required on Cortex-M7 devices that software lock the DWT (e.g. STM32F7)
        cortex_m::peripheral::DWT::unlock();
        core.DWT.enable_cycle_counter();

        cx.schedule
            .blink(cx.start + Duration::from_cycles(SYS_FREQ.0 / 2))
            .unwrap();

        //hprintln!("init::LateResources").unwrap();
        init::LateResources {
            beeper,
            lcd,
            cnt: 0,
        }
    }

    #[idle(resources = [lcd],)]
    fn idle(ctx: idle::Context) -> ! {
        let lcd = ctx.resources.lcd;
        lcd.init().unwrap();

        let mut r = 1u32;
        loop {
            let c = match r % 3 {
                0 => Rgb565::RED,
                1 => Rgb565::GREEN,
                _ => Rgb565::BLUE,
            };

            lcd.clear(c).unwrap();

            // Draw a circle centered around `(32, 32)` with a radius of `10` and a white stroke
            let circle = egcircle!(
                center = (20, 100),
                radius = 10,
                style = primitive_style!(stroke_color = Rgb565::WHITE, stroke_width = 1)
            );
            circle.draw(lcd).unwrap();

            // Create a new text style
            let style = TextStyleBuilder::new(Font6x8)
                .text_color(Rgb565::YELLOW)
                .background_color(Rgb565::BLUE)
                .build();

            // Create a text at position (20, 30) and draw it using the previously defined style
            Text::new("Hello Rust!", Point::new(0, 30))
                .into_styled(style)
                .draw(lcd)
                .unwrap();

            let re = egrectangle!(
                top_left = (10, 50),
                bottom_right = (13, 53),
                style = primitive_style!(
                    stroke_color = Rgb565::WHITE,
                    fill_color = Rgb565::CYAN,
                    stroke_width = 1
                )
            );

            re.draw(lcd).unwrap();

            let rot = Rotation::try_from(r % 4).unwrap();
            lcd.set_rotation(rot).unwrap();
            r += 1;

            asm::delay(SYS_FREQ.0);
        }
    }

    #[task(resources = [beeper, cnt],
           schedule = [blink],
           priority = 1)]
    fn blink(cx: blink::Context) {
        let n = cx.resources.cnt;
        *n += 1;

        //hprintln!("n={}", n).unwrap();
        //cx.resources.lcd_data.odr.write(|w| unsafe { w.bits(*n) });

        cx.schedule
            .blink(cx.scheduled + Duration::from_cycles(SYS_FREQ.0 / 2))
            .unwrap();
    }

    // RTIC requires that unused interrupts are declared in an extern block when
    // using software tasks; these free interrupts will be used to dispatch the
    // software tasks.
    // Full list in  stm32f1::stm32f103::Interrupt
    extern "C" {
        fn EXTI4();
        fn FSMC();
        fn TAMPER();
    }
};
