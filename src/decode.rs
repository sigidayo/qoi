use crate::{
    ColoursToRaw,
    constants::{
        COMPRESSION_TAG_MASK, DIFF_TAG, ENDING_QOI_MAGIC_BYTES, HEADER_LENGTH, INDEX_TAG, LUMA_TAG,
        QOI_MAGIC_BYTES, REMAINING_DATA_MASK, RGB_BYTE, RGBA_BYTE, RUN_TAG,
    },
    error::{
        DecodeError,
        DecodeError::InvalidHeader,
        HeaderError::{
            InvalidColourChannels, InvalidColourSpace, InvalidMagicBytes, MalformedInput,
        },
    },
    model::{ColourChannels, Colourspace, Pixel, PushUnchecked, QoiHeader, SeenPixels},
};

pub fn decode(data: &[u8]) -> Result<Vec<u8>, DecodeError> {
    let header = extract_header(data)?;
    let data = &data[HEADER_LENGTH..data.len() - 8];
    let mut seen = SeenPixels::new();
    let mut previous_pixel = Pixel {
        alpha: 255,
        ..Default::default()
    };

    let mut output_buf: Vec<Pixel> = Vec::new();
    output_buf.reserve_exact((header.height * header.width) as usize);

    let mut cursor = 0;
    while cursor < data.len() {
        let current_byte = data[cursor];
        cursor += 1;

        match current_byte {
            byte if byte == RGB_BYTE => {
                let pixel = Pixel {
                    red: data[cursor],
                    green: data[cursor + 1],
                    blue: data[cursor + 2],
                    alpha: previous_pixel.alpha,
                };
                cursor += 3;
                seen.insert(pixel);
                previous_pixel = pixel;
                unsafe { output_buf.push_unchecked(pixel) }
            }
            byte if byte == RGBA_BYTE => {
                let pixel = Pixel {
                    red: data[cursor],
                    green: data[cursor + 1],
                    blue: data[cursor + 2],
                    alpha: data[cursor + 3],
                };
                cursor += 4;
                seen.insert(pixel);
                previous_pixel = pixel;
                unsafe { output_buf.push_unchecked(pixel) }
            }
            byte => match byte & COMPRESSION_TAG_MASK {
                tag if tag == RUN_TAG => {
                    let count = (byte & REMAINING_DATA_MASK) + 1;
                    for _ in 0..count {
                        unsafe { output_buf.push_unchecked(previous_pixel) }
                    }
                }
                tag if tag == DIFF_TAG => {
                    let dr = ((byte & 0b00110000) >> 4) as i8 - 2;
                    let dg = ((byte & 0b00001100) >> 2) as i8 - 2;
                    let db = (byte & 0b00000011) as i8 - 2;

                    let pixel = Pixel::from_diffs(&previous_pixel, dr, dg, db);
                    seen.insert(pixel);
                    previous_pixel = pixel;
                    unsafe { output_buf.push_unchecked(pixel) }
                }
                tag if tag == LUMA_TAG => {
                    let next_byte = data[cursor];
                    cursor += 1;

                    let dg = (byte & REMAINING_DATA_MASK) as i8 - 32;
                    let dr = dg - 8 + ((next_byte & 0b11110000) >> 4) as i8;
                    let db = dg - 8 + (next_byte & 0b00001111) as i8;

                    let pixel = Pixel::from_diffs(&previous_pixel, dr, dg, db);
                    seen.insert(pixel);
                    previous_pixel = pixel;
                    unsafe { output_buf.push_unchecked(pixel) }
                }
                tag if tag == INDEX_TAG => {
                    let idx = (byte & REMAINING_DATA_MASK) as usize;
                    let pixel = seen.get(idx);
                    previous_pixel = pixel;
                    unsafe { output_buf.push_unchecked(pixel) }
                }
                _ => unreachable!(),
            },
        }
    }

    Ok(output_buf.to_raw())
}

fn extract_header(data: &[u8]) -> Result<QoiHeader, DecodeError> {
    let length = data.len();
    if data[0..4] != QOI_MAGIC_BYTES {
        return Err(InvalidHeader(InvalidMagicBytes {
            expected: "[0x71, 0x6F, 0x69, 0x66]",
            found: format!("{:?}", &data[0..4]),
        }));
    }
    if data[length - 8..length] != ENDING_QOI_MAGIC_BYTES {
        return Err(InvalidHeader(InvalidMagicBytes {
            expected: "[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01]",
            found: format!("{:?}", &data[length - 8..length]),
        }));
    }

    Ok(QoiHeader {
        width: u32::from_be_bytes(data[4..8].try_into().map_err(MalformedInput)?),
        height: u32::from_be_bytes(data[8..12].try_into().map_err(MalformedInput)?),
        channels: match data[12] {
            0x03 => ColourChannels::Rgb,
            0x04 => ColourChannels::Rgba,
            _ => {
                return Err(InvalidHeader(InvalidColourChannels {
                    expected: "0x03 (RGB) or 0x04 (RGBA)",
                    found: format!("{}", data[13]),
                }));
            }
        },
        colorspace: match data[13] {
            0x00 => Colourspace::SRgb,
            0x01 => Colourspace::Linear,
            _ => {
                return Err(InvalidHeader(InvalidColourSpace {
                    expected: "0x00 (SRGB) or 0x01 (Linear)",
                    found: format!("{}", data[13]),
                }));
            }
        },
    })
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use std::{fs, fs::File, io::BufWriter, path::Path};

    use png::Encoder;
    use pretty_assertions::assert_eq;

    use super::*;

    macro_rules! decode {
        ($name:expr, $width:expr, $height:expr) => {
            paste::item! {
                #[test]
                fn [<$name _decode>]() {
                    let data = include_bytes!(concat!("../data/", $name, ".qoi"));
                    let decoded = decode(data).unwrap();
                    assert_eq!($width * $height, decoded.len() / 4);
                    save($name, &decoded, $width, $height);
                }
            }
        };
    }

    fn save(name: &'static str, data: &[u8], w: u32, h: u32) {
        let output_dir = Path::new("test_output");
        if !output_dir.exists() {
            fs::create_dir(output_dir).unwrap();
        }
        let path = output_dir.join(name).with_extension("png");

        let file = File::create(path).unwrap();
        let writer = BufWriter::new(file);

        let mut encoder = Encoder::new(writer, w, h);
        encoder.set_color(png::ColorType::Rgba);
        let mut writer = encoder.write_header().unwrap();

        writer.write_image_data(data).unwrap();
    }

    // decode!("custom", 16, 1);
    decode!("dice", 800, 600);
    decode!("edgecase", 256, 64);
    decode!("kodim10", 512, 768);
    decode!("kodim23", 768, 512);
    decode!("qoi_logo", 448, 220);
    decode!("testcard", 256, 256);
    decode!("testcard_rgba", 256, 256);
    decode!("wikipedia_008", 1152, 858);
}
