//
// LCD init code from
// from https://github.com/adafruit/TFTLCD-Library.git
//
// https://cdn-shop.adafruit.com/datasheets/ILI9328.pdf
//
use core::convert::{Infallible, TryFrom};

use stm32f1xx_hal::{gpio::*, rcc::APB2};

use stm32f1xx_hal::pac::GPIOE;

use cortex_m_semihosting::hprintln;

use embedded_hal::digital::v2::OutputPin;

use embedded_hal::blocking::delay::DelayMs;
use embedded_hal::blocking::delay::DelayUs;

use embedded_graphics::{
    drawable::Pixel,
    geometry::{Point, Size},
    pixelcolor::{raw::RawU16, Rgb565},
    prelude::*,
    primitives::rectangle::*,
    style::{PrimitiveStyle, Styled},
    DrawTarget,
};

/// Screen rotation, CCW
#[derive(Debug, Clone, Copy)]
pub enum Rotation {
    R0,
    R90,
    R180,
    R270,
}

impl TryFrom<u32> for Rotation {
    type Error = LcdError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Rotation::R0),
            1 => Ok(Rotation::R90),
            2 => Ok(Rotation::R180),
            3 => Ok(Rotation::R270),
            _ => Err(LcdError::InvalidRotationId),
        }
    }
}

/// ILI9328
/// https://cdn-shop.adafruit.com/datasheets/ILI9328.pdf
pub struct Lcd<D> {
    delay: D,
    port: GPIOE, // 16b parallel push/pull on port E
    backlight: gpiod::PD14<Output<PushPull>>,
    csn: gpioc::PC8<Output<PushPull>>, //  /CS chip select (inverted)
    rs: gpiod::PD13<Output<PushPull>>, //   RS command/data select
    wrn: gpiob::PB14<Output<PushPull>>, // /WR write signal (inverted)
    rdn: gpiod::PD15<Output<PushPull>>, // /RD read signal (inverted)
    rotation: Rotation,
}

#[derive(Debug, Clone, Copy)]
pub enum LcdError {
    Infallible,
    Init,
    InvalidWindow,
    InvalidRotationId,
}

impl From<Infallible> for LcdError {
    fn from(_: Infallible) -> Self {
        LcdError::Infallible
    }
}

impl<D> DrawTarget<Rgb565> for Lcd<D>
where
    D: DelayMs<u16> + DelayUs<u16>,
{
    type Error = LcdError;

    fn draw_pixel(&mut self, pixel: Pixel<Rgb565>) -> Result<(), Self::Error> {
        let Pixel(p, color) = pixel;
        let lcdp = self.lcd_point(p);

        self.write_register(ILI932XRegister::GramHorAd as u16, lcdp.x as u16)?;
        self.write_register(ILI932XRegister::GramVerAd as u16, lcdp.y as u16)?;

        self.transact(|slf| {
            slf.write_register_index(ILI932XRegister::RwGram as u16)?;
            slf.delay.delay_us(1);
            slf.write_data_word(RawU16::from(color).into_inner())?;
            Ok(())
        })
    }

    fn draw_rectangle(
        &mut self,
        item: &Styled<Rectangle, PrimitiveStyle<Rgb565>>,
    ) -> Result<(), Self::Error> {
        for c in item.style.fill_color {
            self.fill_rectangle(item.primitive, c)?;
        }
        Ok(())
    }

    fn size(&self) -> Size {
        match self.rotation {
            Rotation::R0 => Size::new(TFT_WIDTH as u32, TFT_HEIGHT as u32),
            Rotation::R90 => Size::new(TFT_HEIGHT as u32, TFT_WIDTH as u32),
            Rotation::R180 => Size::new(TFT_WIDTH as u32, TFT_HEIGHT as u32),
            Rotation::R270 => Size::new(TFT_HEIGHT as u32, TFT_WIDTH as u32),
        }
    }
}

const PUSH_PULL_1: u32 = 0b0011;
const PUSH_PULL: u32 = PUSH_PULL_1
    | PUSH_PULL_1
    | PUSH_PULL_1 << 4
    | PUSH_PULL_1 << 8
    | PUSH_PULL_1 << 12
    | PUSH_PULL_1 << 16
    | PUSH_PULL_1 << 20
    | PUSH_PULL_1 << 24
    | PUSH_PULL_1 << 28;

const FLOATING_INPUT_1: u32 = 0b0100;
const FLOATING_INPUT: u32 = FLOATING_INPUT_1
    | FLOATING_INPUT_1
    | FLOATING_INPUT_1 << 4
    | FLOATING_INPUT_1 << 8
    | FLOATING_INPUT_1 << 12
    | FLOATING_INPUT_1 << 16
    | FLOATING_INPUT_1 << 20
    | FLOATING_INPUT_1 << 24
    | FLOATING_INPUT_1 << 28;

const TFT_WIDTH: u16 = 240;
const TFT_HEIGHT: u16 = 320;
const TFT_NATIVE_SIZE: Size = Size::new(TFT_WIDTH as u32, TFT_HEIGHT as u32);

const EM_BGR: u16 = 1 << 12;
const EM_AM: u16 = 1 << 3;
const EM_ID0: u16 = 1 << 4;
const EM_ID1: u16 = 1 << 5;

#[allow(dead_code)]
#[repr(u16)]
enum ILI932XRegister {
    StartOsc = 0x00,
    DrivOutCtrl = 0x01,
    DrivWavCtrl = 0x02,
    EntryMod = 0x03,
    ResizeCtrl = 0x04,
    DispCtrl1 = 0x07,
    DispCtrl2 = 0x08,
    DispCtrl3 = 0x09,
    DispCtrl4 = 0x0a,
    RgbDispIfCtrl1 = 0x0c,
    FrmMarkerPos = 0x0d,
    RgbDispIfCtrl2 = 0x0f,
    PowCtrl1 = 0x10,
    PowCtrl2 = 0x11,
    PowCtrl3 = 0x12,
    PowCtrl4 = 0x13,
    GramHorAd = 0x20,
    GramVerAd = 0x21,
    RwGram = 0x22,
    PowCtrl7 = 0x29,
    FrmRateColCtrl = 0x2b,
    GammaCtrl1 = 0x30,
    GammaCtrl2 = 0x31,
    GammaCtrl3 = 0x32,
    GammaCtrl4 = 0x35,
    GammaCtrl5 = 0x36,
    GammaCtrl6 = 0x37,
    GammaCtrl7 = 0x38,
    GammaCtrl8 = 0x39,
    GammaCtrl9 = 0x3c,
    GammaCtrl10 = 0x3d,
    HorStartAd = 0x50,
    HorEndAd = 0x51,
    VerStartAd = 0x52,
    VerEndAd = 0x53,
    GateScanCtrl1 = 0x60,
    GateScanCtrl2 = 0x61,
    GateScanCtrl3 = 0x6a,
    PartImg1DispPos = 0x80,
    PartImg1StartAd = 0x81,
    PartImg1EndAd = 0x82,
    PartImg2DispPos = 0x83,
    PartImg2StartAd = 0x84,
    PartImg2EndAd = 0x85,
    PanelIfCtrl1 = 0x90,
    PanelIfCtrl2 = 0x92,
    PanelIfCtrl3 = 0x93,
    PanelIfCtrl4 = 0x95,
    PanelIfCtrl5 = 0x97,
    PanelIfCtrl6 = 0x98,
}

impl<D> Lcd<D>
where
    D: DelayMs<u16> + DelayUs<u16>,
{
    pub fn new(
        delay: D,
        port: GPIOE,
        rcc: &mut APB2,

        backlight: gpiod::PD14<Output<PushPull>>,
        csn: gpioc::PC8<Output<PushPull>>,
        rs: gpiod::PD13<Output<PushPull>>,
        wrn: gpiob::PB14<Output<PushPull>>,
        rdn: gpiod::PD15<Output<PushPull>>,
    ) -> Result<Lcd<D>, LcdError> {
        <GPIOE as stm32f1xx_hal::rcc::Enable>::enable(rcc);
        <GPIOE as stm32f1xx_hal::rcc::Reset>::reset(rcc);

        Ok(Lcd {
            delay,
            port,
            backlight,
            csn,
            rs,
            wrn,
            rdn,
            rotation: Rotation::R0,
        })
    }

    pub fn init(&mut self) -> Result<(), LcdError> {
        self.backlight.set_high()?;
        self.output()?;

        self.delay.delay_ms(130);

        let d1 = self.read_register(0)?;

        hprintln!("ID?: {:X}", d1).unwrap();

        self.write_register(ILI932XRegister::StartOsc as u16, 0x0001)?;

        self.delay.delay_ms(50);

        self.write_register(ILI932XRegister::DrivOutCtrl as u16, 0x0100)?;
        self.write_register(ILI932XRegister::DrivWavCtrl as u16, 0x0700)?;

        self.write_register(ILI932XRegister::ResizeCtrl as u16, 0x0000)?;
        self.write_register(ILI932XRegister::DispCtrl2 as u16, 0x0202)?;
        self.write_register(ILI932XRegister::DispCtrl3 as u16, 0x0000)?;
        self.write_register(ILI932XRegister::DispCtrl4 as u16, 0x0000)?;

        self.write_register(ILI932XRegister::RgbDispIfCtrl1 as u16, 0x0)?;

        self.write_register(ILI932XRegister::FrmMarkerPos as u16, 0x0)?;
        self.write_register(ILI932XRegister::RgbDispIfCtrl2 as u16, 0x0)?;
        self.write_register(ILI932XRegister::PowCtrl1 as u16, 0x0000)?;
        self.write_register(ILI932XRegister::PowCtrl2 as u16, 0x0007)?;
        self.write_register(ILI932XRegister::PowCtrl3 as u16, 0x0000)?;
        self.write_register(ILI932XRegister::PowCtrl4 as u16, 0x0000)?;

        self.delay.delay_ms(200);

        self.write_register(ILI932XRegister::PowCtrl1 as u16, 0x1690)?;
        self.write_register(ILI932XRegister::PowCtrl2 as u16, 0x0227)?;

        self.delay.delay_ms(50);

        self.write_register(ILI932XRegister::PowCtrl3 as u16, 0x001a)?;

        self.delay.delay_ms(50);

        self.write_register(ILI932XRegister::PowCtrl4 as u16, 0x1800)?;
        self.write_register(ILI932XRegister::PowCtrl7 as u16, 0x002a)?;

        self.delay.delay_ms(50);

        self.write_register(ILI932XRegister::GammaCtrl1 as u16, 0x0000)?;
        self.write_register(ILI932XRegister::GammaCtrl2 as u16, 0x0000)?;
        self.write_register(ILI932XRegister::GammaCtrl3 as u16, 0x0000)?;
        self.write_register(ILI932XRegister::GammaCtrl4 as u16, 0x0206)?;
        self.write_register(ILI932XRegister::GammaCtrl5 as u16, 0x0808)?;
        self.write_register(ILI932XRegister::GammaCtrl6 as u16, 0x0007)?;
        self.write_register(ILI932XRegister::GammaCtrl7 as u16, 0x0201)?;
        self.write_register(ILI932XRegister::GammaCtrl8 as u16, 0x0000)?;
        self.write_register(ILI932XRegister::GammaCtrl9 as u16, 0x0000)?;
        self.write_register(ILI932XRegister::GammaCtrl10 as u16, 0x0000)?;

        self.set_rotation(Rotation::R0)?;
        self.reset_window()?;

        self.write_register(ILI932XRegister::GramHorAd as u16, 0x0000)?;
        self.write_register(ILI932XRegister::GramVerAd as u16, 0x0000)?;

        self.write_register(ILI932XRegister::GateScanCtrl1 as u16, 0xa700)?;
        self.write_register(ILI932XRegister::GateScanCtrl2 as u16, 0x0003)?;
        self.write_register(ILI932XRegister::GateScanCtrl3 as u16, 0x0000)?;
        self.write_register(ILI932XRegister::PanelIfCtrl1 as u16, 0x0010)?;
        self.write_register(ILI932XRegister::PanelIfCtrl2 as u16, 0x0000)?;
        self.write_register(ILI932XRegister::PanelIfCtrl3 as u16, 0x0003)?;
        self.write_register(ILI932XRegister::PanelIfCtrl4 as u16, 0x1100)?;
        self.write_register(ILI932XRegister::PanelIfCtrl5 as u16, 0x0000)?;
        self.write_register(ILI932XRegister::PanelIfCtrl6 as u16, 0x0000)?;
        self.write_register(ILI932XRegister::DispCtrl1 as u16, 0x0133)?;

        Ok(())
    }

    pub fn set_rotation(&mut self, rotation: Rotation) -> Result<(), LcdError> {
        self.rotation = rotation;

        self.write_register(
            ILI932XRegister::EntryMod as u16,
            (match self.rotation {
                Rotation::R0 => EM_ID0 | EM_ID1,
                Rotation::R90 => EM_AM | EM_ID1,
                Rotation::R180 => 0,
                Rotation::R270 => EM_AM | EM_ID0,
            }) | EM_BGR,
        )
    }

    fn set_window(&mut self, window: Rectangle) -> Result<(), LcdError> {
        let Rectangle {
            top_left,
            bottom_right,
        } = window;
        if top_left.x <= bottom_right.x && top_left.y <= bottom_right.y {
            Ok(())
        } else {
            Err(LcdError::InvalidWindow)
        }?;

        let tl = self.lcd_point(top_left);
        let br = self.lcd_point(bottom_right - Point::new(1, 1));

        let minx = tl.x.min(br.x) as u16;
        let miny = tl.y.min(br.y) as u16;
        let maxx = tl.x.max(br.x) as u16;
        let maxy = tl.y.max(br.y) as u16;

        hprintln!(
            "win: minx: {} miny: {} / maxx: {} maxy: {}",
            minx,
            miny,
            maxx,
            maxy
        )
        .unwrap();

        self.write_register(ILI932XRegister::HorStartAd as u16, minx)?;
        self.write_register(ILI932XRegister::HorEndAd as u16, maxx)?;

        self.write_register(ILI932XRegister::VerStartAd as u16, miny)?;
        self.write_register(ILI932XRegister::VerEndAd as u16, maxy)?;

        self.write_register(ILI932XRegister::GramHorAd as u16, minx)?;
        self.write_register(ILI932XRegister::GramVerAd as u16, miny)?;

        Ok(())
    }

    fn reset_window(&mut self) -> Result<(), LcdError> {
        self.write_register(ILI932XRegister::HorStartAd as u16, 0)?;
        self.write_register(ILI932XRegister::HorEndAd as u16, TFT_WIDTH - 1 as u16)?;

        self.write_register(ILI932XRegister::VerStartAd as u16, 0)?;
        self.write_register(ILI932XRegister::VerEndAd as u16, TFT_HEIGHT - 1 as u16)?;

        Ok(())
    }

    /// Fills a rectangle with a solid color.
    /// Top left / bottom right points included.
    fn fill_rectangle(&mut self, rectangle: Rectangle, color: Rgb565) -> Result<(), LcdError> {
        self.set_window(rectangle)?; // validates input

        let Size { width, height } = rectangle.size();
        let mut n = width * height;

        hprintln!("fill: w: {} h: {} n: {}", width, height, n).unwrap();

        let mut leftover = 0;
        self.transact(|slf| {
            slf.write_register_index(ILI932XRegister::RwGram as u16)?;
            slf.delay.delay_us(1);

            slf.write_data_word(RawU16::from(color).into_inner())?;
            n -= 1;

            // has to be written in 4-words
            leftover = n % 4;
            n += leftover;

            while n > 0 {
                slf.delay.delay_us(1);
                slf.wrn.set_low()?;
                slf.delay.delay_us(1);
                slf.wrn.set_high()?;

                n -= 1;
            }

            Ok(())
        })?;

        self.reset_window()?;

        Ok(())
    }

    pub fn max_btm_right(&self) -> Point {
        let w = TFT_WIDTH as i32 - 1;
        let h = TFT_HEIGHT as i32 - 1;
        match self.rotation {
            Rotation::R0 => Point::new(w, h),
            Rotation::R90 => Point::new(h, w),
            Rotation::R180 => Point::new(w, h),
            Rotation::R270 => Point::new(h, w),
        }
    }

    fn write_register(&mut self, register: u16, data: u16) -> Result<(), LcdError> {
        self.transact(|slf| {
            slf.write_register_index(register)?;
            slf.delay.delay_us(1);
            slf.write_data_word(data)
        })
    }

    fn read_register(&mut self, register: u16) -> Result<u16, LcdError> {
        self.transact(|slf| {
            slf.write_register_index(register)?;
            slf.delay.delay_us(1);
            slf.read_data_word()
        })
    }

    #[allow(dead_code)]
    fn read_port_data(&mut self) -> Result<u16, LcdError> {
        self.transact(|slf| slf.read_data_word())
    }

    fn transact<FT, R>(&mut self, f: FT) -> Result<R, LcdError>
    where
        FT: FnOnce(&mut Lcd<D>) -> Result<R, LcdError>,
    {
        self.rs.set_high()?;
        self.rdn.set_high()?;
        self.wrn.set_high()?;

        self.csn.set_low()?;
        self.delay.delay_us(1);

        let res = f(self);

        self.delay.delay_us(1);
        self.csn.set_high()?;

        res
    }

    fn write_register_index(&mut self, register: u16) -> Result<(), LcdError> {
        self.rs.set_low()?;
        self.write_port_bits(register)?;

        self.wrn.set_low()?;
        self.delay.delay_us(1);
        self.wrn.set_high()?;

        self.rs.set_high()?;
        Ok(())
    }

    fn write_data_word(&mut self, data: u16) -> Result<(), LcdError> {
        self.write_port_bits(data)?;

        self.wrn.set_low()?;
        self.delay.delay_us(1);
        self.wrn.set_high()?;
        Ok(())
    }

    fn read_data_word(&mut self) -> Result<u16, LcdError> {
        self.input()?;

        self.rdn.set_low()?;
        self.delay.delay_us(1);

        let res = self.port.idr.read().bits();

        self.rdn.set_high()?;
        self.output()?;

        Ok(res as u16)
    }

    fn write_port_bits(&mut self, bits: u16) -> Result<(), LcdError> {
        self.port.odr.write(|w| unsafe { w.bits(bits as u32) });
        Ok(())
    }

    /// Enable output on LCD parallel port
    fn output(&mut self) -> Result<(), LcdError> {
        self.port.crl.write(|w| unsafe { w.bits(PUSH_PULL) });
        self.port.crh.write(|w| unsafe { w.bits(PUSH_PULL) });
        Ok(())
    }

    /// Enable floating input on LCD parallel port
    fn input(&mut self) -> Result<(), LcdError> {
        self.port.crl.write(|w| unsafe { w.bits(FLOATING_INPUT) });
        self.port.crh.write(|w| unsafe { w.bits(FLOATING_INPUT) });
        Ok(())
    }

    /// Point in the LCD native coordinates, full screen
    fn lcd_point(&self, p: Point) -> Point {
        self.lcd_window_point(p, TFT_NATIVE_SIZE)
    }

    /// Point in the LCD window native coordinates
    fn lcd_window_point(&self, p: Point, window: Size) -> Point {
        match self.rotation {
            Rotation::R0 => p,
            Rotation::R90 => Point::new(window.width as i32 - 1 - p.y, p.x),
            Rotation::R180 => Point::new(
                window.width as i32 - 1 - p.x,
                window.height as i32 - 1 - p.y,
            ),
            Rotation::R270 => Point::new(p.y, window.height as i32 - 1 - p.x),
        }
    }
}
