#![feature(async_fn_in_trait)]

use ellipsoid::prelude::{winit::window::Window, *};
#[cfg(target_arch="wasm32")]
use wasm_bindgen::prelude::*;


use strum::{Display, EnumIter};

const X_OFFSET: f32 = 0.9;
const RECT_WIDTH: f32 = 0.1;
const RECT_HEIGHT: f32 = 0.6;
const BALL_RADIUS: f32 = 0.05;
const BALL_SPEED: f32 = 1.5;

#[repr(u32)]
#[derive(Clone, Copy, EnumIter, Display, Default)]
#[strum(serialize_all = "snake_case")]
enum PongTextures {
    #[default]
    White,
    Paddle,
    Ball,
}

impl Into<u32> for PongTextures {
    fn into(self) -> u32 {
        self as u32
    }
}

impl Textures for PongTextures {
    fn extension(&self) -> ImageFormat {
        match self {
            Self::White => ImageFormat::Png,
            Self::Paddle => ImageFormat::Jpeg,
            Self::Ball => ImageFormat::Jpeg,
        }
    }
}

struct Ball {
    pos: Vec2,
    dir: Vec2,
}

struct PongGame {
    graphics: Graphics<PongTextures>,
    player_pos: f32,
    enemy_pos: f32,
    ball: Ball,
    up_pressed: bool,
    down_pressed: bool,
}

impl App<PongTextures> for PongGame {
    async fn new(window: Window) -> Self {
        let graphics = Graphics::<PongTextures>::new(window).await;
        Self {
            graphics,
            player_pos: 0.,
            enemy_pos: 0.,
            ball: Ball {
                pos: vec2(0., 0.),
                dir: vec2(rand::random(), rand::random()).normalize(),
            },
            up_pressed: false,
            down_pressed: false
        }
    }
    fn update(&mut self, dt: f32) {
        if self.up_pressed {
            self.player_pos += 1.5 * dt;
        }
        if self.down_pressed {
            self.player_pos -= 1.5 * dt;
        }

        self.ball.pos += self.ball.dir * BALL_SPEED * dt;
        if self.ball.pos.x + BALL_RADIUS > X_OFFSET - RECT_WIDTH * 0.5
            && self.ball.pos.y - BALL_RADIUS < self.enemy_pos + RECT_HEIGHT * 0.5
            && self.ball.pos.y + BALL_RADIUS > self.enemy_pos - RECT_HEIGHT * 0.5
        {
            self.ball.dir.x = -self.ball.dir.x.abs();
        }
        if self.ball.pos.x - BALL_RADIUS < -X_OFFSET + RECT_WIDTH * 0.5
            && self.ball.pos.y - BALL_RADIUS < self.player_pos + RECT_HEIGHT * 0.5
            && self.ball.pos.y + BALL_RADIUS > self.player_pos - RECT_HEIGHT * 0.5
        {
            self.ball.dir.x = self.ball.dir.x.abs();
        }

        if self.ball.pos.y + BALL_RADIUS > 1. {
            self.ball.dir.y = -self.ball.dir.y.abs();
        }
        if self.ball.pos.y - BALL_RADIUS < -1. {
            self.ball.dir.y = self.ball.dir.y.abs();
        }

        if self.ball.pos.x + BALL_RADIUS > 1. || self.ball.pos.x - BALL_RADIUS < -1. {
            self.ball.pos = vec2(0., 0.);
            self.ball.dir = vec2(rand::random(), rand::random()).normalize();
        }

        self.enemy_pos = self.ball.pos.y;
    }
    fn draw(&mut self) {
        let size = self.graphics.window().inner_size();
        let aspect_ratio = size.height as f32 / size.width as f32;

        let gt = GTransform::default(); //.stretch_x(aspect_ratio);

        let background = Shape::from_square_centered().set_color(Color::from_rgb(0.05, 0.04, 0.05)).apply(GTransform::from_inflation(2.));
        
        let player_shape = Shape::from_square().set_texture(PongTextures::Paddle).apply(
            gt.translate(vec2(-X_OFFSET, self.player_pos))
                .stretch(vec2(RECT_WIDTH, RECT_HEIGHT))
                .translate(vec2(-0.5, -0.5)),
        );
        let enemy_shape = Shape::from_square().set_texture(PongTextures::Paddle).apply(
            gt.translate(vec2(X_OFFSET, self.enemy_pos))
                .stretch(vec2(RECT_WIDTH, RECT_HEIGHT))
                .translate(vec2(-0.5, -0.5)),
        );

        let ball_shape = Shape::from_circle(30)
            .apply(gt.translate(self.ball.pos).inflate(BALL_RADIUS))
            .set_texture(PongTextures::Ball);


        self.graphics.add_geometry(background.into());
        self.graphics.add_geometry(player_shape.into());
        self.graphics.add_geometry(enemy_shape.into());
        self.graphics.add_geometry(ball_shape.into());
    }
    fn input(&mut self, event: &WindowEvent) -> bool {
        // when up key is pressed set up_pressed to true
        // when down key is pressed set down_pressed to true

        match event {
            WindowEvent::KeyboardInput { input, .. } => {
                if let Some(keycode) = input.virtual_keycode {
                    match keycode {
                        winit::event::VirtualKeyCode::Up => {
                            self.up_pressed = input.state == winit::event::ElementState::Pressed;
                        }
                        winit::event::VirtualKeyCode::Down => {
                            self.down_pressed = input.state == winit::event::ElementState::Pressed;
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }

        false
    }
    fn graphics_mut(&mut self) -> &mut Graphics<PongTextures> {
        &mut self.graphics
    }
    fn graphics(&self) -> &Graphics<PongTextures> {
        &self.graphics
    }
}


#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn start() {
    ellipsoid::run::<PongTextures, PongGame>().await;
}
