#![no_main]

use libfuzzer_sys::fuzz_target;
use qoi::{Pixel, PushUnchecked};

fuzz_target!(|data: [u8; 4]| {
    let mut v = Vec::with_capacity(1);
    unsafe {
        v.push_unchecked(Pixel {
            red: data[0],
            green: data[1],
            blue: data[2],
            alpha: data[3],
        });
    }
});
