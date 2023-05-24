use super::*;

#[derive(Clone, Copy, Debug)]
pub struct GTransform {
    pub center: Vec2,
    pub rotation: f32,
    pub scale: Vec2,
}

impl Default for GTransform {
    fn default() -> Self {
        Self {
            center: Vec2::ZERO,
            rotation: 0.,
            scale: Vec2::ONE,
        }
    }
}

impl GTransform {
    pub fn from_inflation(inflation: f32) -> Self {
        Self {
            scale: inflation * Vec2::ONE,
            ..Default::default()
        }
    }
    pub fn from_scale(scale: Vec2) -> Self {
        Self {
            scale,
            ..Default::default()
        }
    }
    pub fn from_translation(translation: Vec2) -> Self {
        Self {
            center: translation,
            scale: Vec2::ONE,
            rotation: 0.,
        }
    }
    pub fn inflate(mut self, mp: f32) -> Self {
        self.scale *= mp;
        self
    }
    pub fn inflate_fixed(mut self, dp: f32) -> Self {
        self.scale += dp * Vec2::ONE;
        self
    }
    pub fn stretch(mut self, scale: Vec2) -> Self {
        self.scale *= scale;
        self
    }
    pub fn stretch_x(mut self, scale_x: f32) -> Self {
        self.scale.x *= scale_x;
        self
    }
    pub fn stretch_fixed(mut self, size: Vec2) -> Self {
        self.scale += size;
        self
    }
    pub fn rotate(mut self, radians: f32) -> Self {
        self.rotation += radians;
        self
    }
    pub fn set_scale(mut self, scale: Vec2) -> Self {
        self.scale = scale;
        self
    }
    pub fn translate(mut self, translation: Vec2) -> Self {
        self.center += Vec2::from_angle(self.rotation).rotate(translation) * self.scale;
        self
    }
    /// Rotation is still applied tho
    pub fn translate_fixed(mut self, translation: Vec2) -> Self {
        self.center += Vec2::from_angle(self.rotation).rotate(translation);
        self
    }
    pub fn set_scale_y(mut self, scale_y: f32) -> Self {
        self.scale.y = scale_y;
        self
    }
    pub fn transform(&self, point: Vec2) -> Vec2 {
        Vec2::from_angle(self.rotation).rotate(point * self.scale) + self.center
    }
    pub fn inv_transform(&self, point: Vec2) -> Vec2 {
        Vec2::from_angle(-self.rotation).rotate(point - self.center) / self.scale
    }
}
