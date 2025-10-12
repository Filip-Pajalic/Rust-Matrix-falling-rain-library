use macroquad::prelude::*;

/// Batch renderer for matrix characters
/// Collects all draw data and renders in fewer draw calls
pub struct BatchRenderer {
    // Pre-allocated buffers for batch rendering
    positions: Vec<Vec2>,
    colors: Vec<Color>,
    glyphs: Vec<char>,
    capacity: usize,
}

impl BatchRenderer {
    /// Create a new batch renderer with expected capacity
    pub fn new(capacity: usize) -> Self {
        BatchRenderer {
            positions: Vec::with_capacity(capacity),
            colors: Vec::with_capacity(capacity),
            glyphs: Vec::with_capacity(capacity),
            capacity,
        }
    }

    /// Add a character to the batch
    #[inline]
    pub fn push(&mut self, position: Vec2, color: Color, glyph: char) {
        self.positions.push(position);
        self.colors.push(color);
        self.glyphs.push(glyph);
    }

    /// Clear the batch for the next frame
    #[inline]
    pub fn clear(&mut self) {
        self.positions.clear();
        self.colors.clear();
        self.glyphs.clear();
    }

    /// Render all batched characters
    pub fn render(&self, font: &Font, font_size: f32) {
        // Render all characters in one go
        for i in 0..self.positions.len() {
            let mut buf = [0u8; 4];
            let glyph_str = self.glyphs[i].encode_utf8(&mut buf);
            
            draw_text_ex(
                glyph_str,
                self.positions[i].x,
                self.positions[i].y,
                TextParams {
                    font_size: font_size as u16,
                    font: Some(font),
                    color: self.colors[i],
                    ..Default::default()
                },
            );
        }
    }

    /// Get current batch size
    #[inline]
    pub fn len(&self) -> usize {
        self.positions.len()
    }

    /// Check if batch is empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.positions.is_empty()
    }
}