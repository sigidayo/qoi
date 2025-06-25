#![no_main]

use libfuzzer_sys::fuzz_target;
use qoi::{Pixel, ColoursToRaw, RawToColours};

fuzz_target!(|data: &[u8]| {
    let mapped = data
        .chunks(4)
        .map(|i| Pixel {
            red: *i.get(0).unwrap_or_else(|| &0_u8),
            green: *i.get(1).unwrap_or_else(|| &0_u8),
            blue: *i.get(2).unwrap_or_else(|| &0_u8),
            alpha: *i.get(3).unwrap_or_else(|| &0_u8),
        })
        .collect::<Vec<_>>();

    let unrolled = unroll(&mapped);
    let alloc_skip = mapped.to_raw();

    assert_eq!(unrolled, alloc_skip);
    assert_eq!(unrolled, unroll(&alloc_skip.to_colours())); 
});

fn unroll(v: &Vec<Pixel>) -> Vec<u8> {
    v
        .iter()
        .cloned()
        .map(|i| vec![i.red, i.green, i.blue, i.alpha])
        .flatten()
        .collect::<Vec<_>>()
}