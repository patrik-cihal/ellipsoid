use ellipsoid::prelude::*;

const X_OFFSET: f32 = 0.9;
const RECT_WIDTH: f32 = 0.1;
const RECT_HEIGHT: f32 = 0.6;
const BALL_RADIUS: f32 = 0.05;
const BALL_SPEED: f32 = 1.5;

struct Ball {
    pos: Vec2,
    dir: Vec2,
}

struct PongGame {
    graphics: Graphics,
    player_pos: f32,
    enemy_pos: f32,
    ball: Ball
}

impl App for PongGame {
    fn new(graphics: Graphics) -> Self {
        Self { graphics, player_pos: 0., enemy_pos: 0., ball: Ball { pos: vec2(0., 0.), dir: vec2(rand::random(), rand::random()).normalize()} }
    }
    fn update(&mut self, dt: f32) {
        self.ball.pos += self.ball.dir * BALL_SPEED * dt;
        if self.ball.pos.x + BALL_RADIUS > X_OFFSET-RECT_WIDTH*0.5 && self.ball.pos.y-BALL_RADIUS < self.enemy_pos + RECT_HEIGHT*0.5 && self.ball.pos.y + BALL_RADIUS > self.enemy_pos - RECT_HEIGHT * 0.5 {
            self.ball.dir.x = -self.ball.dir.x.abs();
        }
        if self.ball.pos.x - BALL_RADIUS < -X_OFFSET+RECT_WIDTH*0.5 && self.ball.pos.y-BALL_RADIUS < self.player_pos + RECT_HEIGHT*0.5 && self.ball.pos.y + BALL_RADIUS > self.player_pos - RECT_HEIGHT * 0.5 {
            self.ball.dir.x = self.ball.dir.x.abs();
        }

        if self.ball.pos.y+BALL_RADIUS > 1. {
            self.ball.dir.y = -self.ball.dir.y.abs();
        }
        if self.ball.pos.y-BALL_RADIUS < -1. {
            self.ball.dir.y = self.ball.dir.y.abs();
        }

        if self.ball.pos.x+BALL_RADIUS > 1. || self.ball.pos.x-BALL_RADIUS < -1. {
            self.ball.pos = vec2(0., 0.);
            self.ball.dir = vec2(rand::random(), rand::random()).normalize();
        }

        self.enemy_pos = self.ball.pos.y;
    }
    fn draw(&mut self) {
        let player_shape = Shape::from_square().apply(GTransform::from_translation(vec2(-X_OFFSET, self.player_pos)).stretch(vec2(RECT_WIDTH, RECT_HEIGHT)).translate(vec2(-0.5, -0.5)));
        let enemy_shape = Shape::from_square().apply(GTransform::from_translation(vec2(X_OFFSET, self.enemy_pos)).stretch(vec2(RECT_WIDTH, RECT_HEIGHT)).translate(vec2(-0.5, -0.5)));

        let ball_shape = Shape::from_circle(30).apply(GTransform::from_translation(self.ball.pos).inflate(BALL_RADIUS));

        self.graphics.add_geometry(player_shape.into());
        self.graphics.add_geometry(enemy_shape.into());
        self.graphics.add_geometry(ball_shape.into());

    }
    fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput { input, .. } => {
                if let Some(key) = input.virtual_keycode {
                    match key {
                        winit::event::VirtualKeyCode::W => {
                            self.player_pos += 0.1;
                        }
                        winit::event::VirtualKeyCode::S => {
                            self.player_pos -= 0.1;
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
        false
    }
    fn graphics_mut(&mut self) -> &mut Graphics {
        &mut self.graphics
    }
    fn graphics(&self) -> &Graphics {
        &self.graphics
    }
}

fn main() {
    async_std::task::block_on(ellipsoid::run::<PongGame>());
}
