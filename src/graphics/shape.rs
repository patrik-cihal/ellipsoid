use glam::vec2;

use super::*;

#[derive(Clone)]
pub struct Shape {
    pub points: Vec<Vec2>,
    color: Color,
}

impl Shape {
    pub fn new(points: Vec<Vec2>) -> Self {
        Self { points, color: Color::WHITE }
    }
    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
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
        Self::new(
            vec![
                Vec2::new(0., 0.),
                Vec2::new(1., 0.),
                Vec2::new(1., 1.),
                Vec2::new(0., 1.),
            ],
        )
    }
    pub fn from_triangle() -> Self {
        Self::new(
            vec![vec2(0., 0.5), vec2(-0.5, -0.5), vec2(0.5, -0.5)],
        )
    }
    pub fn from_line(length: f32, thickness: f32) -> Self {
        let square = Self::from_square();
        // length += thickness;

        let gtransform = GTransform::from_translation(-Vec2::Y/2. * thickness)
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
    pub fn apply(mut self, gtransform: GTransform) -> Shape {
        for point in &mut self.points {
            *point = gtransform.transform(*point);
        }
        self
    }
}

#[derive(Default, Clone, Copy, Debug)]
pub struct GTransform {
    pub center: Vec2,
    pub rotation: f32,
    pub scale: Vec2,
}

impl GTransform {
    pub fn from_inflation(inflation: f32) -> Self {
        Self {
            scale: inflation * Vec2::ONE,
            ..Default::default()
        }
    }
    pub fn from_scale(scale: Vec2) -> Self {
        Self { scale, ..Default::default() }
    }
    pub fn from_translation(translation: Vec2) -> Self {
        Self {
            center: translation,
            scale: Vec2::ONE,
            rotation: 0.
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
        Vec2::from_angle(self.rotation).rotate(point*self.scale)
            + self.center
    }
}

impl Into<(Vec<Vertex>, Vec<u32>)> for Shape {
    fn into(self) -> (Vec<Vertex>, Vec<u32>) {
        let mut vertices: Vec<Vertex> = vec![self.points[0].into(), self.points[1].into()];
        let mut indices = vec![];

        for i in 2..self.points.len() {
            vertices.push(self.points[i].into());
            indices.push(0);
            indices.push(i as u32 - 1);
            indices.push(i as u32);
        }
        
        for vertex in &mut vertices {
            vertex.color = self.color.into();
        }

        (vertices, indices)
    }
}
