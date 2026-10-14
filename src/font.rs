use std::collections::HashMap;

pub const FONT_CHAR_WIDTH: usize = 8;
pub const FONT_CHAR_HEIGHT: usize = 13;
pub const DISPLAY_WIDTH: usize = 200;
pub const DISPLAY_HEIGHT: usize = 65;
pub const DED_ROWS: usize = 5;
pub const DED_COLUMNS: usize = 24;

const FONT_MAP: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890\x01()<>[]+-\x02/=^|~\x7f.,!?:;&_'\"%#@{} \x7f\x7f\x7f\x7f";

pub struct Font {
    glyphs: HashMap<u8, [bool; FONT_CHAR_WIDTH * FONT_CHAR_HEIGHT]>,
    glyphs_inverted: HashMap<u8, [bool; FONT_CHAR_WIDTH * FONT_CHAR_HEIGHT]>,
}

impl Font {
    pub fn load() -> Self {
        let normal_bmp = include_bytes!("../font/ded_font.bmp");
        let inverted_bmp = include_bytes!("../font/ded_font_inverted.bmp");

        let mut glyphs = HashMap::new();
        let mut glyphs_inverted = HashMap::new();

        let normal_pixels = extract_pixels(normal_bmp);
        let inverted_pixels = extract_pixels(inverted_bmp);

        for (i, &key) in FONT_MAP.iter().enumerate() {
            if key == 0x7f && i < FONT_MAP.len() - 4 {
                continue;
            }

            let normal_glyph = get_glyph_pixels(&normal_pixels, i);
            let inverted_glyph = get_glyph_pixels(&inverted_pixels, i);

            glyphs.insert(key, normal_glyph);
            glyphs_inverted.insert(key, inverted_glyph);

            if key == 0x02 {
                glyphs.insert(b'*', normal_glyph);
                glyphs_inverted.insert(b'*', inverted_glyph);
            }
        }

        Self {
            glyphs,
            glyphs_inverted,
        }
    }

    pub fn get_glyph(&self, c: u8, inverted: bool) -> &[bool; FONT_CHAR_WIDTH * FONT_CHAR_HEIGHT] {
        let map = if inverted {
            &self.glyphs_inverted
        } else {
            &self.glyphs
        };
        map.get(&c)
            .or_else(|| map.get(&0x7f))
            .unwrap_or(&[false; FONT_CHAR_WIDTH * FONT_CHAR_HEIGHT])
    }
}

fn extract_pixels(bmp: &[u8]) -> Vec<bool> {
    // BMP offset 10 contains the starting address of the image data
    let data_offset = u32::from_le_bytes(bmp[10..14].try_into().unwrap()) as usize;
    let width = u32::from_le_bytes(bmp[18..22].try_into().unwrap()) as usize;
    let height = u32::from_le_bytes(bmp[22..26].try_into().unwrap()) as usize;

    // BMP rows are padded to 4 bytes.
    // 8-bit BMP, so 1 byte per pixel.
    let row_size = (width + 3) & !3;
    let mut pixels = vec![false; width * height];

    for y in 0..height {
        let row_start = data_offset + (height - 1 - y) * row_size; // BMP is bottom-up
        for x in 0..width {
            let pixel_val = bmp[row_start + x];
            // In the C# code, it used GetBrightness() > 0.
            // For these specific BMPs, it's likely that non-zero is "on".
            pixels[y * width + x] = pixel_val > 0;
        }
    }
    pixels
}

fn get_glyph_pixels(pixels: &[bool], index: usize) -> [bool; FONT_CHAR_WIDTH * FONT_CHAR_HEIGHT] {
    let mut glyph = [false; FONT_CHAR_WIDTH * FONT_CHAR_HEIGHT];
    let total_width = 576; // 72 * 8
    for y in 0..FONT_CHAR_HEIGHT {
        for x in 0..FONT_CHAR_WIDTH {
            let src_x = index * FONT_CHAR_WIDTH + x;
            glyph[y * FONT_CHAR_WIDTH + x] = pixels[y * total_width + src_x];
        }
    }
    glyph
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_font_load() {
        let font = Font::load();
        assert!(font.glyphs.contains_key(&b'A'));
        assert!(font.glyphs.contains_key(&b'1'));
        assert!(font.glyphs.contains_key(&b'*'));
        assert!(font.glyphs.contains_key(&0x7f));

        let a_glyph = font.get_glyph(b'A', false);
        // 'A' should have some pixels on
        assert!(a_glyph.iter().any(|&p| p));
    }
}
