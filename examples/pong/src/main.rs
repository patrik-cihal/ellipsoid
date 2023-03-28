use ellipsoid::{Graphics, App, run, WindowEvent};

struct MyApp {
    graphics: Graphics
}

impl App for MyApp {
    fn new(graphics: Graphics) -> Self {
        Self { graphics }
    }
    fn update(&mut self, dt: f32) {
        println!("Update: {}", dt);
    }
    fn draw(&mut self) {
        for i in 0..10000 {
        }
    }
    fn input(&mut self, event: &WindowEvent) -> bool {
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
    async_std::task::block_on(run::<MyApp>());
}
