use glam::{vec2, vec3};

use super::*;

#[derive(Clone, Debug)]
pub struct Shape<T: Textures> {
    pub points: Vec<(Vec2, Vec2)>,
    texture: T,
    color: Color,
    z: f32,
}

impl<T: Textures> Shape<T> {
    pub fn new(points: Vec<Vec2>) -> Self {
        Self {
            points: points.into_iter().map(|p| (p, Vec2::ZERO)).collect(),
            texture: Default::default(),
            color: Color::WHITE,
            z: 0.,
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
    pub fn from_square_centered() -> Self {
        Self::new(vec![
            Vec2::new(-0.5, -0.5),
            Vec2::new(0.5, -0.5),
            Vec2::new(0.5, 0.5),
            Vec2::new(-0.5, 0.5),
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
        let mut left_lower_point = Vec2::new(std::f32::MAX, std::f32::MAX);
        let mut right_upper_point = Vec2::new(std::f32::MIN, std::f32::MIN);

        for (point, _) in &self.points {
            left_lower_point = left_lower_point.min(*point);
            right_upper_point = right_upper_point.max(*point);
        }

        for (point, tex_coord) in &mut self.points {
            *tex_coord = (*point - left_lower_point) / (right_upper_point - left_lower_point);
        }

        self
    }

    pub fn set_texture(mut self, t: T) -> Self {
        self.texture = t;
        self
    }
    
    pub fn reset_texture(mut self) -> Self {
        self.texture = T::default();
        self
    }

    pub fn set_color(mut self, c: Color) -> Self {
        self.color = c;
        self
    }

    pub fn set_z(mut self, z: f32) -> Self {
        self.z = z;
        self
    }
}

impl<T: Textures> Into<(Vec<Vertex<T>>, Vec<u32>)> for Shape<T> {
    fn into(self) -> (Vec<Vertex<T>>, Vec<u32>) {
        let points = self
            .points
            .into_iter()
            .map(|(p, tc)| (vec3(p.x, p.y, self.z), tc))
            .collect::<Vec<_>>();
        let mut vertices: Vec<Vertex<T>> = vec![points[0].into(), points[1].into()];
        let mut indices = vec![];

        for i in 2..points.len() {
            vertices.push(points[i].into());
            indices.push(0);
            indices.push(i as u32 - 1);
            indices.push(i as u32);
        }

        for vertex in &mut vertices {
            vertex.texture = self.texture;
            vertex.color = self.color;
        }

        (vertices, indices)
    }
}
