use std::{fmt::Display, marker::PhantomData, num::NonZeroU32, path::Path};

use glam::{Vec2, Vec3};

use strum::{EnumIter, IntoEnumIterator};
use winit::window::Window;

mod gtransform;
mod shape;

pub use gtransform::GTransform;
pub use shape::Shape;

mod color;
pub use color::Color;

const VERTEX_BUFFER_INIT_SIZE: wgpu::BufferAddress =
    1000 * std::mem::size_of::<VertexRaw>() as wgpu::BufferAddress;
const INDEX_BUFFER_INIT_SIZE: wgpu::BufferAddress =
    300 * std::mem::size_of::<u32>() as wgpu::BufferAddress;

pub trait Textures: IntoEnumIterator + Display + Default + Into<u32> + Copy {
    fn name(&self) -> String {
        self.to_string()
    }
    fn extension(&self) -> image::ImageFormat {
        image::ImageFormat::Png
    }
    fn folder() -> &'static str {
        "assets/textures"
    }
}

pub type Geometry<T> = (Vec<Vertex<T>>, Vec<u32>);

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Vertex<T: Textures> {
    position: Vec3,
    texture: T,
    texture_coords: Vec2,
    color: Color,
}

impl<T: Textures> Into<Vertex<T>> for (Vec3, Vec2) {
    fn into(self) -> Vertex<T> {
        Vertex {
            position: self.0,
            texture: T::default(),
            texture_coords: self.1,
            color: Color::WHITE,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable, Default)]
pub struct VertexRaw {
    position: [f32; 3],
    texture_index: u32,
    texture_coords: [f32; 2],
    color: [f32; 4],
}

impl VertexRaw {
    const ATTRIBS: [wgpu::VertexAttribute; 4] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Uint32, 2 => Float32x2, 3 => Float32x4];

    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;

        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

impl<T: Textures> Into<VertexRaw> for Vertex<T> {
    fn into(self) -> VertexRaw {
        VertexRaw {
            position: [self.position.x, self.position.y, self.position.z],
            texture_index: self.texture.into(),
            texture_coords: [self.texture_coords.x, self.texture_coords.y],
            color: self.color.into(),
        }
    }
}

fn align<T: Default + Clone>(v: &mut Vec<T>) {
    let len = v.len();
    let rem = len % 4;
    if rem > 0 {
        v.extend(std::iter::repeat(T::default()).take(4 - rem));
    }
}

pub struct Graphics<T: Textures> {
    pub size: winit::dpi::PhysicalSize<u32>,
    pub egui_platform: egui_winit_platform::Platform,
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
    window: Window,
    egui_rpass: egui_wgpu_backend::RenderPass,
    start_time: chrono::NaiveTime,
    vertices: Vec<Vertex<T>>,
    indices: Vec<u32>,
    texture_views: Vec<wgpu::TextureView>,
    textures_bind_group: wgpu::BindGroup,
    depth_texture: wgpu::Texture,
    depth_texture_view: wgpu::TextureView,
}

impl<T: Textures> Graphics<T> {
    pub async fn new(window: Window) -> Self {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
        });

        // # Safety
        //
        // The surface needs to live as long as the window that created it.
        // State owns the window so this should be safe.
        let surface = unsafe { instance.create_surface(&window) }.unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::TEXTURE_BINDING_ARRAY | wgpu::Features::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING,
                    // WebGL doesn't support all of wgpu's features, so if
                    // we're building for the web we'll have to disable some.
                    limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                },
                None, // Trace path
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        // Shader code in this tutorial assumes an Srgb surface texture. Using a different
        // one will result all the colors comming out darker. If you want to support non
        // Srgb surfaces, you'll need to account for that when drawing to the frame.
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .filter(|f| f.describe().srgb)
            .next()
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        let texture_views = T::iter()
            .enumerate()
            .map(|(i, texture)| {
                let mut ext = None;
                for new_ext in texture.extension().extensions_str() {
                    let path = Path::new(&T::folder()).join(format!(
                        "{}.{}",
                        texture.name(),
                        new_ext
                    ));
                    if path.exists() {
                        ext = Some(new_ext);
                        break;
                    }
                }
                let Some(ext) = ext else {
                    panic!("Texture {} not found", texture);
                };

                let path = Path::new(&T::folder()).join(format!("{}.{}", texture.name(), ext));

                let image_bytes = std::fs::read(path).unwrap();
                let diffuse_image =
                    image::load_from_memory_with_format(&image_bytes, texture.extension()).unwrap();

                use image::GenericImageView;
                let dimensions = diffuse_image.dimensions();

                let texture_size = wgpu::Extent3d {
                    width: dimensions.0,
                    height: dimensions.1,
                    depth_or_array_layers: 1,
                };

                let format = wgpu::TextureFormat::Rgba8UnormSrgb;

                let diffuse_texture = device.create_texture(&wgpu::TextureDescriptor {
                    size: texture_size,
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format,
                    usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
                    label: Some(format!("diffuse_texture_{}", i).as_str()),
                    view_formats: &[],
                });

                let diffuse_rgba = diffuse_image.to_rgba8().into_raw();

                let bytes_per_pixel = format.describe().block_size as u32;
                let bytes_per_row = dimensions.0 * bytes_per_pixel;

                queue.write_texture(
                    wgpu::ImageCopyTexture {
                        texture: &diffuse_texture,
                        mip_level: 0,
                        origin: wgpu::Origin3d::ZERO,
                        aspect: wgpu::TextureAspect::All,
                    },
                    &diffuse_rgba,
                    wgpu::ImageDataLayout {
                        offset: 0,
                        bytes_per_row: std::num::NonZeroU32::new(bytes_per_row),
                        rows_per_image: std::num::NonZeroU32::new(dimensions.1),
                    },
                    texture_size,
                );

                diffuse_texture.create_view(&wgpu::TextureViewDescriptor::default())
            })
            .collect::<Vec<_>>();

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("texture_bind_group_layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: NonZeroU32::new(texture_views.len() as u32),
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());

        let texture_views_ref = texture_views.iter().collect::<Vec<_>>();
        let textures_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureViewArray(&texture_views_ref),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            layout: &texture_bind_group_layout,
            label: Some("texture_bind_group"),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&texture_bind_group_layout],
                push_constant_ranges: &[],
            });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });


        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("depth_texture"),
            size: wgpu::Extent3d {
                width: config.width,
                height: config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth24PlusStencil8,
            view_formats: &[],
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        });

        let depth_texture_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let depth_stencil_state = wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth24PlusStencil8,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        };        

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[VertexRaw::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                // Setting this to anything other than Fill requires Features::POLYGON_MODE_LINE
                // or Features::POLYGON_MODE_POINT
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: Some(depth_stencil_state),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            // If the pipeline will be used with a multiview render pass, this
            // indicates how many array layers the attachments will have.
            multiview: None,
        });

        let vertex_buffer_desc = wgpu::BufferDescriptor {
            label: Some("vertex_buffer"),
            size: VERTEX_BUFFER_INIT_SIZE,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        };

        let vertex_buffer = device.create_buffer(&vertex_buffer_desc);

        let index_buffer_desc = wgpu::BufferDescriptor {
            label: Some("index_buffer"),
            size: INDEX_BUFFER_INIT_SIZE,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        };

        let index_buffer = device.create_buffer(&index_buffer_desc);

        let num_indices = 0;

        let egui_platform =
            egui_winit_platform::Platform::new(egui_winit_platform::PlatformDescriptor {
                physical_width: size.width as u32,
                physical_height: size.height as u32,
                scale_factor: window.scale_factor(),
                font_definitions: egui::FontDefinitions::default(),
                style: Default::default(),
            });

        let egui_rpass = egui_wgpu_backend::RenderPass::new(&device, surface_format, 1);

        Self {
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            num_indices,
            window,
            egui_platform,
            egui_rpass,
            start_time: chrono::Local::now().time(),
            vertices: vec![],
            indices: vec![],
            textures_bind_group,
            texture_views,
            depth_texture,
            depth_texture_view
        }
    }

    pub fn add_geometry(&mut self, geometry: Geometry<T>) {
        let index_offset = self.vertices.len() as u32;

        let (vertices, indices) = geometry;

        self.vertices.extend(vertices);
        self.indices
            .extend(indices.into_iter().map(|i| i + index_offset));
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width == 0 || new_size.height == 0 {
            return;
        }
        self.size = new_size;
        self.config.width = new_size.width;
        self.config.height = new_size.height;
        self.surface.configure(&self.device, &self.config);

        self.depth_texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("depth_texture"),
            size: wgpu::Extent3d {
                width: self.config.width,
                height: self.config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth24PlusStencil8,
            view_formats: &[],
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        });

        self.depth_texture_view = self.depth_texture.create_view(&wgpu::TextureViewDescriptor::default());
    }

    pub fn handle_raw_event(&mut self, event: &winit::event::Event<()>) {
        self.egui_platform.handle_event(event);
    }

    pub fn update(&mut self) {
        self.egui_platform.update_time(
            (chrono::Local::now().time() - self.start_time).num_milliseconds() as f64 / 1000.0,
        );
        self.egui_platform.begin_frame();

        self.num_indices = self.indices.len() as u32;

        let mut vertices_raw = std::mem::take(&mut self.vertices)
            .into_iter()
            .map(|x| x.into())
            .collect::<Vec<VertexRaw>>();

        align(&mut self.indices);
        align(&mut vertices_raw);

        if self.vertex_buffer.size()
            < (vertices_raw.len() * std::mem::size_of::<VertexRaw>()) as u64
        {
            let mut new_size = self.vertex_buffer.size();
            while new_size < (vertices_raw.len() * std::mem::size_of::<VertexRaw>()) as u64 {
                new_size *= 2;
            }
            self.vertex_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("vertex_buffer"),
                size: new_size,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }

        if self.index_buffer.size() < (self.indices.len() * std::mem::size_of::<u32>()) as u64 {
            let mut new_size = self.index_buffer.size();
            while new_size < (self.indices.len() * std::mem::size_of::<u32>()) as u64 {
                new_size *= 2;
            }
            self.index_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("index_buffer"),
                size: new_size,
                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }

        self.queue
            .write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&vertices_raw));
        self.queue
            .write_buffer(&self.index_buffer, 0, bytemuck::cast_slice(&self.indices));

        self.indices.clear();
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let full_output = self.egui_platform.end_frame(Some(&self.window));
        let paint_jobs = self.egui_platform.context().tessellate(full_output.shapes);

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.1,
                            b: 0.1,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            render_pass.set_pipeline(&self.render_pipeline);

            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

            render_pass.set_bind_group(0, &self.textures_bind_group, &[]);

            render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
        }

        let tdelta = full_output.textures_delta;
        self.egui_rpass
            .add_textures(&self.device, &self.queue, &tdelta)
            .expect("Failed to add textures");
        let screen_descriptor = egui_wgpu_backend::ScreenDescriptor {
            physical_width: self.size.width,
            physical_height: self.size.height,
            scale_factor: self.window.scale_factor() as f32,
        };
        self.egui_rpass
            .update_buffers(&self.device, &self.queue, &paint_jobs, &screen_descriptor);

        self.egui_rpass
            .execute(&mut encoder, &view, &paint_jobs, &screen_descriptor, None)
            .unwrap();

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        self.egui_rpass
            .remove_textures(tdelta)
            .expect("Failed to remove textures");

        Ok(())
    }
}
