#![feature(async_fn_in_trait)]

use ellipsoid::prelude::*;
use strum::{Display, EnumIter};

struct MyApp {
    graphics: Graphics<MyTextures>,
    rotation: f32,
}

#[derive(Debug, Clone, Copy, EnumIter, Display, Default)]
#[strum(serialize_all = "snake_case")]
enum MyTextures {
    #[default]
    White,
}

impl Into<u32> for MyTextures {
    fn into(self) -> u32 {
        self as u32
    }
}

impl Textures for MyTextures {}

impl App<MyTextures> for MyApp {
    async fn new(window: winit::window::Window) -> Self {
        let graphics = Graphics::<MyTextures>::new(window).await;
        MyApp {
            graphics,
            rotation: 0.,
        }
    }

    fn update(&mut self, dt: f32) {
        self.rotation += dt;
    }

    fn draw(&mut self) {
        let triangle = Shape::from_triangle()
            .set_color(Color::GREEN)
            .apply(GTransform::default().rotate(self.rotation));
        self.graphics.add_geometry(triangle.into());
    }

    fn input(&mut self, _event: &WindowEvent) -> bool {
        false
    }

    fn graphics_mut(&mut self) -> &mut Graphics<MyTextures> {
        &mut self.graphics
    }

    fn graphics(&self) -> &Graphics<MyTextures> {
        &self.graphics
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn start() {
    ellipsoid::run::<MyTextures, MyApp>().await;
}
