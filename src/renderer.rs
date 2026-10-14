use crate::font::{
    DED_COLUMNS, DED_ROWS, DISPLAY_HEIGHT, DISPLAY_WIDTH, FONT_CHAR_HEIGHT, FONT_CHAR_WIDTH, Font,
};

pub struct Renderer {
    font: Font,
}

impl Renderer {
    pub fn new() -> Self {
        Self { font: Font::load() }
    }

    pub fn render(&self, lines: &[[u8; 26]; 5], inverted: &[[u8; 26]; 5]) -> Vec<u8> {
        let mut buffer = vec![false; DISPLAY_WIDTH * DISPLAY_HEIGHT];

        for row in 0..DED_ROWS {
            let line = &lines[row];
            let inv_line = &inverted[row];

            for col in 0..DED_COLUMNS {
                let char_byte = line[col];
                let inv_byte = inv_line[col];
                let is_inverted = inv_byte != b' ' && inv_byte != 0;

                let glyph = self.font.get_glyph(char_byte, is_inverted);

                draw_glyph(&mut buffer, row, col, glyph);
            }
        }

        pack_into_bit_buffer(buffer)
    }
}

fn draw_glyph(buffer: &mut [bool], row: usize, col: usize, glyph: &[bool; 104]) {
    let start_x = col * FONT_CHAR_WIDTH;
    let start_y = row * FONT_CHAR_HEIGHT;

    for gy in 0..FONT_CHAR_HEIGHT {
        for gx in 0..FONT_CHAR_WIDTH {
            if glyph[gy * FONT_CHAR_WIDTH + gx] {
                let x = start_x + gx;
                let y = start_y + gy;
                if x < DISPLAY_WIDTH && y < DISPLAY_HEIGHT {
                    buffer[y * DISPLAY_WIDTH + x] = true;
                }
            }
        }
    }
}

fn pack_into_bit_buffer(buffer: Vec<bool>) -> Vec<u8> {
    let packed_size = (DISPLAY_WIDTH * DISPLAY_HEIGHT) / 8;
    let mut packed = vec![0u8; packed_size + 4];
    // 4 bytes header/padding? C# adds 4.

    for row in 0..DISPLAY_HEIGHT {
        for col in 0..(DISPLAY_WIDTH / 8) {
            let mut b = 0u8;
            for i in 0..8 {
                let x = col * 8 + i;
                if buffer[row * DISPLAY_WIDTH + x] {
                    b |= 1 << i;
                }
            }
            packed[4 + row * (DISPLAY_WIDTH / 8) + col] = b;
        }
    }
    packed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_renderer_blank() {
        let renderer = Renderer::new();
        let lines = [[b' '; 26]; 5];
        let inverted = [[b' '; 26]; 5];
        let data = renderer.render(&lines, &inverted);
        assert_eq!(data.len(), (200 * 65 / 8) + 4);
        // All pixels should be 0 (except maybe the 4-byte header if we put something there, but we don't)
        assert!(data.iter().all(|&b| b == 0));
    }

    #[test]
    fn test_renderer_with_text() {
        let renderer = Renderer::new();
        let mut lines = [[b' '; 26]; 5];
        lines[0][0] = b'A';
        let inverted = [[b' '; 26]; 5];
        let data = renderer.render(&lines, &inverted);
        // Should not be all zero
        assert!(data.iter().any(|&b| b != 0));
    }
}
