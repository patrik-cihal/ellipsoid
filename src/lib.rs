use std::time;

use serde::{Deserialize, Serialize};
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{WindowBuilder},
};
pub use winit::event::WindowEvent;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

mod graphics;
pub use graphics::{Geometry, Graphics, GTransform, Shape};

pub trait App {
    fn new(graphics: Graphics) -> Self;
    fn graphics(&self) -> &Graphics;
    fn graphics_mut(&mut self) -> &mut Graphics;
    fn input(&mut self, _event: &WindowEvent) -> bool {
        false
    }
    fn update(&mut self, dt: f32);
    fn draw(&mut self);
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn run<A: App + 'static>() {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Warn).expect("Could't initialize logger");
        } else {
            env_logger::init();
        }
    }

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    #[cfg(target_arch = "wasm32")]
    {
        // Winit prevents sizing with CSS, so we have to set
        // the size manually when on web.
        use winit::dpi::PhysicalSize;
        window.set_inner_size(PhysicalSize::new(1280, 720));

        use winit::platform::web::WindowExtWebSys;
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| {
                let dst = doc.get_element_by_id("spacecraft")?;
                let canvas = web_sys::Element::from(window.canvas());
                dst.append_child(&canvas).ok()?;
                Some(())
            })
            .expect("Couldn't append canvas to document body.");
    }

    let mut app = A::new(Graphics::new(window).await);

    let mut last_update = now();

    event_loop.run(move |event, _, control_flow| {
        app.graphics_mut().handle_raw_event(&event);
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == app.graphics().window().id() => {
                if !app.input(event) {
                    match event {
                        WindowEvent::CloseRequested
                        | WindowEvent::KeyboardInput {
                            input:
                                KeyboardInput {
                                    state: ElementState::Pressed,
                                    virtual_keycode: Some(VirtualKeyCode::Escape),
                                    ..
                                },
                            ..
                        } => *control_flow = ControlFlow::Exit,
                        WindowEvent::Resized(physical_size) => {
                            app.graphics_mut().resize(*physical_size);
                        }
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            // new_inner_size is &mut so w have to dereference it twice
                            app.graphics_mut().resize(**new_inner_size);
                        }
                        _ => {}
                    }
                }
            }
            Event::RedrawRequested(window_id) if window_id == app.graphics().window().id() => {
                let now = now();
                let dt = (now-last_update).as_secs_f32();
                last_update = now;

                app.graphics_mut().update();
                app.update(dt);
                app.draw();

                match app.graphics_mut().render() {
                    Ok(_) => {}
                    // Reconfigure the surface if it's lost or outdated
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                        let size = app.graphics().size;
                        app.graphics_mut().resize(size)
                    }
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    // We're ignoring timeouts
                    Err(wgpu::SurfaceError::Timeout) => log::warn!("Surface timeout"),
                }
            }
            Event::MainEventsCleared => {
                // RedrawRequested will only trigger once, unless we manually
                // request it.
                app.graphics().window().request_redraw();
            }
            _ => {}
        }
    });
}


#[derive(Serialize, Deserialize)]
pub struct Interval {
    last: time::Duration,
    interval: time::Duration,
}

impl Interval {
    pub fn new(interval: time::Duration) -> Self {
        Self {
            last: now(),
            interval,
        }
    }
    pub fn check(&mut self) -> bool {
        let now  = now();
        if now-self.last > self.interval {
            self.last = now;
            true
        } else {
            false
        }
    }
}

pub fn now() -> time::Duration {
    time::Duration::from_millis(chrono::Local::now().timestamp_millis() as u64)
}