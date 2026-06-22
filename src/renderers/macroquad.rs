use macroquad::prelude::*;

use crate::{GlyphInstance, MatrixRain, MatrixRenderer, Rgba};

pub struct MacroquadRenderer<'font> {
    font: &'font Font,
    font_size_px: u16,
}

impl<'font> MacroquadRenderer<'font> {
    pub fn new(font: &'font Font, font_size_px: u16) -> Self {
        Self { font, font_size_px }
    }

    pub fn render(&mut self, rain: &MatrixRain) {
        self.render_glyphs(rain.glyphs());
    }
}

impl MatrixRenderer for MacroquadRenderer<'_> {
    fn render_glyphs(&mut self, glyphs: &[GlyphInstance]) {
        for glyph in glyphs {
            let mut buffer = [0; 4];
            let glyph_text = glyph.glyph.encode_utf8(&mut buffer);

            draw_text_ex(
                glyph_text,
                glyph.position.x,
                glyph.position.y + self.font_size_px as f32,
                TextParams {
                    font: Some(self.font),
                    font_size: self.font_size_px,
                    color: color_from_rgba(glyph.color),
                    ..Default::default()
                },
            );
        }
    }
}

fn color_from_rgba(color: Rgba) -> Color {
    Color::from_rgba(color.r, color.g, color.b, color.a)
}
