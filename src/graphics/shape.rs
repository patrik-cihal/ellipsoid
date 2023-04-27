use glam::vec2;

use super::*;

#[derive(Clone)]
pub struct Shape<T: Textures> {
    pub points: Vec<(Vec2, Vec2)>,
    texture: T,
}

impl<T: Textures> Shape<T> {
    pub fn new(points: Vec<Vec2>) -> Self {
        Self {
            points: points.into_iter().map(|p| (p, Vec2::ZERO)).collect(),
            texture: Default::default(),
        }
        .update_texture_coords()
    }

    pub fn from_circle(segments: usize) -> Self {
        let mut points = Vec::with_capacity(segments);
        for i in 0..segments {
            let angle = (i as f32 / segments as f32) * 2. * std::f32::consts::PI;
            points.push(Vec2::new(angle.cos(), angle.sin()));
        }
        Self::new(points)
    }
    pub fn from_square() -> Self {
        Self::new(vec![
            Vec2::new(0., 0.),
            Vec2::new(1., 0.),
            Vec2::new(1., 1.),
            Vec2::new(0., 1.),
        ])
    }
    pub fn from_triangle() -> Self {
        Self::new(vec![vec2(0., 0.5), vec2(-0.5, -0.5), vec2(0.5, -0.5)])
    }
    pub fn from_line(length: f32, thickness: f32) -> Self {
        let square = Self::from_square();
        // length += thickness;

        let gtransform = GTransform::from_translation(-Vec2::Y / 2. * thickness)
            .stretch(vec2(length, thickness));

        square.apply(gtransform)
    }
    pub fn from_polygon(sides: usize) -> Self {
        let mut points = Vec::with_capacity(sides);
        for i in 0..sides {
            let angle = (i as f32 / sides as f32) * 2. * std::f32::consts::PI;
            points.push(Vec2::new(angle.cos(), angle.sin()));
        }
        Self::new(points)
    }

    /// Rotation is after scale
    pub fn apply(mut self, gtransform: GTransform) -> Shape<T> {
        for (point, _) in &mut self.points {
            *point = gtransform.transform(*point);
        }
        self
    }

    pub fn update_texture_coords(mut self) -> Self {
        let mut left_upper_point = Vec2::new(std::f32::MAX, std::f32::MAX);
        let mut right_lower_point = Vec2::new(std::f32::MIN, std::f32::MIN);

        for (point, _) in &self.points {
            left_upper_point = left_upper_point.min(*point);
            right_lower_point = right_lower_point.max(*point);
        }

        for (point, tex_coord) in &mut self.points {
            *tex_coord =
                (*point - left_upper_point) / (right_lower_point - left_upper_point).length();
        }

        self
    }

    pub fn set_texture(mut self, t: T) -> Self {
        self.texture = t;
        self
    }
}

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
    pub fn stretch(mut self, scale: Vec2) -> Self {
        self.scale *= scale;
        self
    }
    pub fn stretch_x(mut self, scale_x: f32) -> Self {
        self.scale.x *= scale_x;
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

impl<T: Textures> Into<(Vec<Vertex<T>>, Vec<u32>)> for Shape<T> {
    fn into(self) -> (Vec<Vertex<T>>, Vec<u32>) {
        let mut vertices: Vec<Vertex<T>> = vec![self.points[0].into(), self.points[1].into()];
        let mut indices = vec![];

        for i in 2..self.points.len() {
            vertices.push(self.points[i].into());
            indices.push(0);
            indices.push(i as u32 - 1);
            indices.push(i as u32);
        }

        for vertex in &mut vertices {
            vertex.texture = self.texture;
        }

        (vertices, indices)
    }
}
