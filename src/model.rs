use thiserror::Error;

use crate::model::sealed::Sealed;

mod sealed {
    pub trait Sealed {}
}

pub trait ColoursToRaw: Sealed {
    fn to_raw(self) -> Vec<u8>;
}
pub trait RawToColours: Sealed {
    fn to_colours(self) -> Vec<Pixel>;
}

#[derive(Debug, Error)]
pub enum DecodeError {
    #[error("invalid header (expected {expected:?}, found {found:?})")]
    InvalidHeader { expected: String, found: String },
    #[error("{0}")]
    GenericError(#[from] Box<dyn std::error::Error>),
    #[error("{0}")]
    IoError(#[from] std::io::Error),
}

#[derive(Debug)]
pub struct QoiHeader {
    pub width: u32,
    pub height: u32,
    pub channels: ColourChannels,
    pub colorspace: Colourspace,
}

#[derive(Debug)]
pub enum ColourChannels {
    RGB,
    RGBA,
}

#[derive(Debug)]
pub enum Colourspace {
    SRGB,
    Linear,
}

#[derive(Default, Debug, Clone, Copy)]
pub struct Pixel {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: u8,
}

impl Sealed for Vec<u8> {}
impl Sealed for Vec<Pixel> {}

impl ColoursToRaw for Vec<Pixel> {
    fn to_raw(mut self) -> Vec<u8> {
        // Sanity check
        assert_eq!(size_of::<Pixel>(), 4);

        let length = self.len() * 4; // There are 4 fields per ColourPoint
        let capacity = self.capacity() * 4;

        let ptr = self.as_mut_ptr() as *mut u8;

        // Drop `s` without running deconstructor so that there is no user after free in our new vec
        core::mem::forget(self);

        // SAFETY:
        // - ColourPoint contains 4 u8s which can be represented as [u8; 4]
        // - The allocation is not used elsewhere. ColourPoint(s) is forgotten
        unsafe { Vec::from_raw_parts(ptr, length, capacity) }
    }
}
impl RawToColours for Vec<u8> {
    fn to_colours(mut self) -> Vec<Pixel> {
        // Same implementation as above

        assert_eq!(size_of::<Pixel>(), 4);

        let length = self.len() / 4;
        let capacity = self.capacity() / 4;

        let ptr = self.as_mut_ptr() as *mut Pixel;

        core::mem::forget(self);

        unsafe { Vec::from_raw_parts(ptr, length, capacity) }
    }
}

impl Pixel {
    pub fn from_diffs(previous_pixel: &Self, dr: i8, dg: i8, db: i8) -> Self {
        Self {
            red: (previous_pixel.red as i8).wrapping_add(dr) as u8,
            green: (previous_pixel.green as i8).wrapping_add(dg) as u8,
            blue: (previous_pixel.blue as i8).wrapping_add(db) as u8,
            alpha: previous_pixel.alpha,
        }
    }

    #[inline]
    pub fn index_position(&self) -> usize {
        ((self.red as u16 * 3
            + self.green as u16 * 5
            + self.blue as u16 * 7
            + self.alpha as u16 * 11)
            % 64) as usize
    }
}
