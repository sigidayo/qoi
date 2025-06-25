use crate::model::{
    ColourChannels, Colourspace, DecodeError,
    DecodeError::{GenericError, InvalidHeader},
    QoiHeader,
};
pub use crate::model::{ColoursToRaw, Pixel, RawToColours};

mod model;

const HEADER_LENGTH: usize = 14;
const MAGIC_BYTES: [u8; 4] = [0x71, 0x6F, 0x69, 0x66]; // "qoif"
const ENDING_MAGIC_BYTES: [u8; 8] = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01];

const RGB_TAG: u8 = 0b11111110;
const RGBA_TAG: u8 = 0b11111111;

const COMPRESSION_TAG_MASK: u8 = 0b11000000;
const REMAINING_DATA_MASK: u8 = 0b00111111;
const INDEX_TAG: u8 = 0b00000000;
const DIFF_TAG: u8 = 0b01000000;
const LUMA_TAG: u8 = 0b10000000;
const RUN_TAG: u8 = 0b11000000;

pub fn decode(data: &[u8]) -> Result<Vec<u8>, DecodeError> {
    let header = extract_header(data)?;
    let mut data = data[HEADER_LENGTH..data.len() - 8].into_iter();
    let mut seen: [Option<Pixel>; 64] = [None; 64];
    let mut previous_pixel = Pixel {
        alpha: 255,
        ..Default::default()
    };
    let mut output_buf: Vec<Pixel> =
        Vec::with_capacity((header.height * header.width) as usize);

    while let Some(value) = data.next() {
        match value {
            x if x == &RGB_TAG => {
                let pixel = Pixel {
                    red: *data.next().unwrap(),
                    green: *data.next().unwrap(),
                    blue: *data.next().unwrap(),
                    alpha: previous_pixel.alpha,
                };
                seen[pixel.index_position()] = Some(pixel);
                previous_pixel = pixel;
                output_buf.push(pixel);
            }
            x if x == &RGBA_TAG => {
                let pixel = Pixel {
                    red: *data.next().unwrap(),
                    green: *data.next().unwrap(),
                    blue: *data.next().unwrap(),
                    alpha: *data.next().unwrap(),
                };
                seen[pixel.index_position()] = Some(pixel);
                previous_pixel = pixel;
                output_buf.push(pixel);
            }
            x => match x & COMPRESSION_TAG_MASK {
                x if x == RUN_TAG => {
                    let count = (x & REMAINING_DATA_MASK) + 1;
                    for _ in 0..count {
                        output_buf.push(previous_pixel);
                    }
                }
                x if x == DIFF_TAG => {
                    let dr = ((x & 0b00110000) >> 4) as i8 - 2;
                    let dg = ((x & 0b00001100) >> 2) as i8 - 2;
                    let db = (x & 0b00000011) as i8 - 2;

                    let pixel = Pixel::from_diffs(&previous_pixel, dr, dg, db);
                    seen[pixel.index_position()] = Some(pixel);
                    previous_pixel = pixel;
                    output_buf.push(pixel);
                }
                x if x == LUMA_TAG => {
                    let next_byte = data
                        .next()
                        .expect("There should be another byte after the luma tag");

                    let dg = (x & REMAINING_DATA_MASK) as i8 - 32;
                    let dr = dg - 8 + ((next_byte & 0b11110000) >> 4) as i8;
                    let db = dg - 8 + (next_byte & 0b00001111) as i8;

                    let pixel = Pixel::from_diffs(&previous_pixel, dr, dg, db);
                    seen[pixel.index_position()] = Some(pixel);
                    previous_pixel = pixel;
                    output_buf.push(pixel);
                }
                x if x == INDEX_TAG => {
                    let idx = (x & REMAINING_DATA_MASK) as usize;
                    match seen.get(idx) {
                        Some(Some(pixel)) => {
                            previous_pixel = *pixel;
                            output_buf.push(*pixel)
                        }
                        Some(None) => {
                            let pixel = Pixel::default();
                            seen[pixel.index_position()] = Some(pixel);
                            previous_pixel = pixel;
                            output_buf.push(pixel);
                        }
                        None => unreachable!()
                    }
                }
                _ => unreachable!()
            }
        }
    }

    Ok(output_buf.to_raw())
}

fn extract_header(data: &[u8]) -> Result<QoiHeader, DecodeError> {
    let length = data.len();
    if !(data[0..4] == MAGIC_BYTES) {
        return Err(InvalidHeader {
            expected: format!("magic bytes {MAGIC_BYTES:?}"),
            found: format!("{:?}", &data[0..4]),
        });
    }
    if !(data[length - 8..length] == ENDING_MAGIC_BYTES) {
        return Err(InvalidHeader {
            expected: format!("magic bytes {ENDING_MAGIC_BYTES:?}"),
            found: format!("{:?}", &data[length - 8..length]),
        });
    }

    Ok(QoiHeader {
        width: u32::from_be_bytes(
            data[4..8]
                .try_into()
                .map_err(|e| GenericError(Box::new(e)))?,
        ),
        height: u32::from_be_bytes(
            data[8..12]
                .try_into()
                .map_err(|e| GenericError(Box::new(e)))?,
        ),
        channels: match data[12] {
            0x03 => ColourChannels::RGB,
            0x04 => ColourChannels::RGBA,
            _ => {
                return Err(InvalidHeader {
                    expected: "0x03 (RGB) or 0x04 (RGBA)".to_string(),
                    found: format!("{}", data[13]),
                });
            }
        },
        colorspace: match data[13] {
            0x00 => Colourspace::SRGB,
            0x01 => Colourspace::Linear,
            _ => {
                return Err(InvalidHeader {
                    expected: "0x00 (SRGB) or 0x01 (Linear)".to_string(),
                    found: format!("{}", data[13]),
                });
            }
        },
    })
}

#[cfg(test)]
mod tests {
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
                    assert_eq!(decoded.len() / 4, $width * $height);
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
        let path = output_dir
            .join(name)
            .with_extension("png");

        let file = File::create(path).unwrap();
        let writer = BufWriter::new(file);

        let mut encoder = Encoder::new(writer, w, h);
        encoder.set_color(png::ColorType::Rgba);
        let mut writer = encoder.write_header().unwrap();

        writer.write_image_data(&data).unwrap();
    }

    decode!("dice", 800, 600);
    decode!("edgecase", 256, 64);
    decode!("kodim10", 512, 768);
    decode!("kodim23", 768, 512);
    decode!("qoi_logo", 448, 220);
    decode!("testcard", 256, 256);
    decode!("testcard_rgba", 256, 256);
    decode!("wikipedia_008", 1152, 858);
}
