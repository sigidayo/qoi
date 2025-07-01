use core::ptr;

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

pub trait PushUnchecked<T>: Sealed {
    /// Will push an item and not check if there is enough capacity
    ///
    /// # Safety
    /// Caller must ensure the array has enough capacity to hold `T`.
    unsafe fn push_unchecked(&mut self, value: T);
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
    Rgb,
    Rgba,
}

#[derive(Debug)]
pub enum Colourspace {
    SRgb,
    Linear,
}

#[derive(Default, Debug, Clone, Copy)]
pub struct Pixel {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: u8,
}

#[derive(Debug)]
pub struct SeenPixels {
    inner: [Pixel; 64],
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

impl PushUnchecked<Pixel> for Vec<Pixel> {
    #[inline(always)]
    unsafe fn push_unchecked(&mut self, value: Pixel) {
        let end = self.as_mut_ptr().add(self.len());
        ptr::write(end, value);
        self.set_len(self.len() + 1);
    }
}

impl Pixel {
    #[inline(always)]
    pub fn from_diffs(previous_pixel: &Self, dr: i8, dg: i8, db: i8) -> Self {
        Self {
            red: (previous_pixel.red as i8).wrapping_add(dr) as u8,
            green: (previous_pixel.green as i8).wrapping_add(dg) as u8,
            blue: (previous_pixel.blue as i8).wrapping_add(db) as u8,
            alpha: previous_pixel.alpha,
        }
    }

    #[inline(always)]
    pub fn index_position(&self) -> usize {
        ((self.red as u16 * 3
            + self.green as u16 * 5
            + self.blue as u16 * 7
            + self.alpha as u16 * 11)
            % 64) as usize
    }
}

impl SeenPixels {
    pub fn new() -> Self {
        // Precomputed index of 53 for 0,0,0,255 https://github.com/phoboslab/qoi/issues/258 for why we do this
        let mut inner = [Pixel::default(); 64];
        inner[52] = Pixel {
            alpha: 255,
            ..Default::default()
        };
        Self { inner }
    }

    pub fn get(&self, idx: usize) -> Pixel {
        self.inner[idx]
    }

    pub fn insert(&mut self, pixel: Pixel) -> usize {
        let idx = pixel.index_position();
        self.inner[idx] = pixel;
        idx
    }
}
