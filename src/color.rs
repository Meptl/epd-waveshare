#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Color {
    Black,
    White,
}

impl From<u8> for Color {
    fn from(value: u8) -> Self {
        match value & 0x01 {
            0x00 => Color::Black,
            0x01 => Color::White,
            _ => unreachable!(),
        }
    }
}

impl Into<u8> for Color {
    fn into(self) -> u8 {
        match self {
            Color::White => 0xff,
            Color::Black => 0x00,
        }
    }
}

#[cfg(feature = "graphics")]
use embedded_graphics::prelude::*;
#[cfg(feature = "graphics")]
impl PixelColor for Color {}

