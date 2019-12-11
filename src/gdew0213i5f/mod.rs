//! A simple Driver for the GDEW0213I5F 2.13" E-Ink Display via SPI
//!
//! Untested!
//!
//! # Example for the 2.9 in E-Ink Display
//!
//! ```rust,ignore
//! use epd_waveshare::{
//!     gdew0213i5f::{GDEW0213I5F, Display2in9},
//!     graphics::{Display, DisplayRotation},
//!     prelude::*,
//! };
//! use embedded_graphics::Drawing;
//!
//! // Setup EPD
//! let mut epd = GDEW0213I5F::new(&mut spi, cs_pin, busy_in, dc, rst, &mut delay).unwrap();
//!
//! // Use display graphics
//! let mut display = Display2in9::default();
//!
//! // Write some hello world in the screenbuffer
//! display.draw(
//!     Font6x8::render_str("Hello World!")
//!         .stroke(Some(Color::Black))
//!         .fill(Some(Color::White))
//!         .translate(Coord::new(5, 50))
//!         .into_iter(),
//! );
//!
//! // Display updated frame
//! epd.update_frame(&mut spi, &display.buffer()).unwrap();
//! epd.display_frame(&mut spi).expect("display frame new graphics");
//!
//! // Set the EPD to sleep
//! epd.sleep(&mut spi).expect("sleep");
//! ```

pub const WIDTH: u32 = 104;
pub const HEIGHT: u32 = 212;
// pub const DPI: u16 = 111;
// pub const PU_DELAY = 100; // ms between partial updates.
pub const DEFAULT_BACKGROUND_COLOR: Color = Color::White;
const IS_BUSY_LOW: bool = false;

use embedded_hal::{
    blocking::{delay::*, spi::Write},
};

use crate::color::Color;

use crate::traits::*;

use crate::interface::{InputPin, OutputPin, DisplayInterface};

mod command;
use command::Command;

mod constants;

// #[cfg(feature = "graphics")]
// mod graphics;
// #[cfg(feature = "graphics")]
// pub use self::gdew0213i5f::graphics::Display2in9;

/// GDEW0213I5F driver
///
pub struct GDEW0213I5F<SPI, CS, BUSY, DC, RST> {
    /// SPI
    interface: DisplayInterface<SPI, CS, BUSY, DC, RST>,
    /// Color
    background_color: Color,
    /// Refresh LUT
    refresh: RefreshLUT,
}

impl<SPI, CS, BUSY, DC, RST> GDEW0213I5F<SPI, CS, BUSY, DC, RST>
where
    SPI: Write<u8>,
    CS: OutputPin,
    BUSY: InputPin,
    DC: OutputPin,
    RST: OutputPin,
{
    fn init<DELAY: DelayMs<u8>>(
        &mut self,
        spi: &mut SPI,
        delay: &mut DELAY,
    ) -> Result<(), SPI::Error> {
        self.interface.reset(delay);

        self.interface.cmd_with_data(spi, Command::POWER_SETTING, &[0x03, 0x00, 0x02b, 0x2b, 0x03])?;
        self.interface.cmd_with_data(spi, Command::BOOSTER_SOFT_START, &[0x17, 0x17, 0x17])?;
        self.interface.cmd(spi, Command::POWER_ON)?;

        self.wait_until_idle();

        self.interface.cmd_with_data(spi, Command::PANEL_SETTING, &[0xbf, 0x0d])?;
        self.interface.cmd_with_data(spi, Command::PLL_CONTROL, &[0x3a])?;
        self.interface.cmd_with_data(spi, Command::RESOLUTION_SETTING, &[WIDTH as u8, (HEIGHT >> 8) as u8, HEIGHT as u8])?;

        Ok(())
    }

    fn wait_until_idle(&mut self) {
        self.interface.wait_until_idle(IS_BUSY_LOW);
    }
}

impl<SPI, CS, BUSY, DC, RST> WaveshareDisplay<SPI, CS, BUSY, DC, RST>
    for GDEW0213I5F<SPI, CS, BUSY, DC, RST>
where
    SPI: Write<u8>,
    CS: OutputPin,
    BUSY: InputPin,
    DC: OutputPin,
    RST: OutputPin,
{
    fn width(&self) -> u32 {
        WIDTH
    }

    fn height(&self) -> u32 {
        HEIGHT
    }

    fn new<DELAY: DelayMs<u8>>(
        spi: &mut SPI,
        cs: CS,
        busy: BUSY,
        dc: DC,
        rst: RST,
        delay: &mut DELAY,
    ) -> Result<Self, SPI::Error> {
        let mut epd = GDEW0213I5F {
            interface: DisplayInterface::new(cs, busy, dc, rst),
            background_color: DEFAULT_BACKGROUND_COLOR,
            refresh: RefreshLUT::FULL,
        };

        epd.init(spi, delay)?;

        Ok(epd)
    }

    fn sleep(&mut self, spi: &mut SPI) -> Result<(), SPI::Error> {
        self.interface.cmd(spi, Command::POWER_OFF)?;
        self.wait_until_idle();
        self.interface.cmd_with_data(spi, Command::DEEP_SLEEP, &[0xA5])?;
        Ok(())
    }

    fn wake_up<DELAY: DelayMs<u8>>(
        &mut self,
        spi: &mut SPI,
        delay: &mut DELAY,
    ) -> Result<(), SPI::Error> {
        self.init(spi, delay)?;

        self.wait_until_idle();
        Ok(())
    }

    fn update_frame(&mut self, spi: &mut SPI, buffer: &[u8]) -> Result<(), SPI::Error> {
        self.set_lut(spi, Some(RefreshLUT::FULL))?;
        self.interface.cmd(spi, Command::DISPLAY_START_TRANSMISSION_1)?;
        self.interface.data_x_times(spi, self.background_color.into(), WIDTH / 8 * HEIGHT)?;
        self.interface.cmd_with_data(spi, Command::DISPLAY_START_TRANSMISSION_2, buffer)?;
        self.wait_until_idle();
        Ok(())
    }

    fn update_partial_frame(
        &mut self,
        spi: &mut SPI,
        buffer: &[u8],
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    ) -> Result<(), SPI::Error> {
        self.set_lut(spi, Some(RefreshLUT::QUICK))?;
        self.interface.cmd(spi, Command::PARTIAL_IN)?;
        // Screen height is 16 bits whilst width is 8 bits. Since the max height is 212, the top 8
        // bits of height should always be 0x00. But just to be formally correct, lets do the all
        // the mutations.
        self.interface.cmd_with_data(
            spi,
            Command::PARTIAL_WINDOW,
            &[
                x as u8,
                (x + width) as u8,
                (y >> 8) as u8,
                y as u8,
                ((y + height) >> 8) as u8,
                (y + height) as u8,
                0x01
            ]
        )?;
        self.interface.cmd_with_data(spi, Command::DISPLAY_START_TRANSMISSION_2, buffer)?;
        Ok(())
    }

    fn display_frame(&mut self, spi: &mut SPI) -> Result<(), SPI::Error> {
        self.interface.cmd(spi, Command::DISPLAY_REFRESH)?;

        if self.refresh == RefreshLUT::QUICK {
            self.interface.cmd(spi, Command::PARTIAL_OUT)?;
        }

        // Deserves a delay here to prevent screen degradation.

        self.wait_until_idle();
        Ok(())
    }

    fn clear_frame(&mut self, spi: &mut SPI) -> Result<(), SPI::Error> {
        self.set_lut(spi, Some(RefreshLUT::FULL))?;
        let color = self.background_color.into();

        self.interface.cmd(spi, Command::DISPLAY_START_TRANSMISSION_2)?;
        self.interface.data_x_times(spi, color, WIDTH / 8 * HEIGHT)?;

        Ok(())
    }

    fn set_background_color(&mut self, background_color: Color) {
        self.background_color = background_color;
    }

    fn background_color(&self) -> &Color {
        &self.background_color
    }

    fn set_lut(
        &mut self,
        spi: &mut SPI,
        refresh_rate: Option<RefreshLUT>,
    ) -> Result<(), SPI::Error> {
        if let Some(refresh_lut) = refresh_rate {
            self.refresh = refresh_lut;
        }
        match self.refresh {
            RefreshLUT::FULL => {
                self.interface.cmd_with_data(spi, Command::VCM_DC_SETTING, &[0x08])?;
                self.interface.cmd_with_data(spi, Command::VCOM_AND_DATA_INTERVAL_SETTING, &[0x97])?;

                self.interface.cmd_with_data(spi, Command::VCOM_LUT, &constants::LUT_VCOM_DC)?;
                self.interface.cmd_with_data(spi, Command::W2W_LUT, &constants::LUT_WW)?;
                self.interface.cmd_with_data(spi, Command::B2W_LUT, &constants::LUT_BW)?;
                self.interface.cmd_with_data(spi, Command::W2B_LUT, &constants::LUT_WB)?;
                self.interface.cmd_with_data(spi, Command::B2B_LUT, &constants::LUT_BB)?;
            },
            RefreshLUT::QUICK => {
                self.interface.cmd_with_data(spi, Command::VCM_DC_SETTING, &[0x08])?;
                self.interface.cmd_with_data(spi, Command::VCOM_AND_DATA_INTERVAL_SETTING, &[0x47])?;

                self.interface.cmd_with_data(spi, Command::VCOM_LUT, &constants::LUT_VCOM_DC_PARTIAL)?;
                self.interface.cmd_with_data(spi, Command::W2W_LUT, &constants::LUT_WW_PARTIAL)?;
                self.interface.cmd_with_data(spi, Command::B2W_LUT, &constants::LUT_BW_PARTIAL)?;
                self.interface.cmd_with_data(spi, Command::W2B_LUT, &constants::LUT_WB_PARTIAL)?;
                self.interface.cmd_with_data(spi, Command::B2B_LUT, &constants::LUT_BB_PARTIAL)?;
            },
        }
        Ok(())
    }

    fn is_busy(&self) -> bool {
        self.interface.is_busy(IS_BUSY_LOW)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn epd_size() {
        assert_eq!(WIDTH, 104);
        assert_eq!(HEIGHT, 212);
        assert_eq!(DEFAULT_BACKGROUND_COLOR, Color::White);
    }
}
