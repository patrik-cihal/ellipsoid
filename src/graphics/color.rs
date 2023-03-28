
#[derive(Copy, Clone)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const WHITE: Color = Color {r: 1.0, g: 1.0, b: 1.0, a: 1.0, };
    pub const BLACK: Color = Color {r: 0., g: 0., b: 0., a: 1.0, };
    pub const GREEN: Color = Color {r: 0., g: 1.0, b: 0., a: 1.0, };
    pub const RED: Color = Color {r: 1.0, g: 0., b: 0., a: 1.0, };

    pub fn from_rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }
    pub fn from_hex(hex: u32) -> Self {
        let r = ((hex >> 16) & 0xFF) as f32 / 255.0;
        let g = ((hex >> 8) & 0xFF) as f32 / 255.0;
        let b = (hex & 0xFF) as f32 / 255.0;
        let a = ((hex >> 24) & 0xFF) as f32 / 255.0;
        Self { r, g, b, a }
    }
    pub fn set_alpha(mut self, a: f32) -> Self {
        self.a = a;
        self
    }
}

impl Into<Color> for [f32; 3] {
    fn into(self) -> Color {
        Color {
            r: self[0],
            g: self[1],
            b: self[2],
            a: 1.0,
        }
    }
}

impl Into<[f32; 4]> for Color {
    fn into(self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }
}