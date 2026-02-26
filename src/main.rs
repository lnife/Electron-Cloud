use nalgebra_glm as glm;
use std::io::{self, Write};
use std::sync::{Arc, Mutex};
use wgpu::util::DeviceExt;
use winit::{
    event::*,
    event_loop::EventLoop,
    keyboard::{Key, NamedKey},
    window::{Window, WindowBuilder},
};

mod camera;
mod geometry;
mod physics;
mod texture;

use camera::Camera;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniform {
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    fn new() -> Self {
        Self {
            view_proj: glm::Mat4::identity().into(),
        }
    }

    fn update_view_proj(&mut self, camera: &Camera, projection: &glm::Mat4) {
        self.view_proj = (projection * camera.get_view_matrix()).into();
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct InstanceRaw {
    position: [f32; 3],
    color: [f32; 4],
}

struct State<'a> {
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    window: &'a Window,
    camera: Arc<Mutex<Camera>>,
    camera_uniform: CameraUniform,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    render_pipeline: wgpu::RenderPipeline,
    sphere_vertex_buffer: wgpu::Buffer,
    num_sphere_vertices: u32,
    instance_buffer: wgpu::Buffer,
    num_instances: u32,
    depth_view: wgpu::TextureView,
}

impl<'a> State<'a> {
    async fn new(window: &'a Window, num_particles: usize) -> Self {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        let surface = instance.create_surface(window).unwrap();
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
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    label: None,
                },
                None,
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let depth_view = texture::create_depth_texture(&device, &config, "depth_texture");

        let camera = Arc::new(Mutex::new(Camera::new(glm::vec3(0.0, 0.0, 0.0), 30.0)));
        let mut camera_uniform = CameraUniform::new();
        let projection = glm::perspective_zo(
            size.width as f32 / size.height as f32,
            glm::radians(&glm::vec1(45.0))[0],
            0.1,
            100.0,
        );
        camera_uniform.update_view_proj(&camera.lock().unwrap(), &projection);

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("camera_bind_group_layout"),
            });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&camera_bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[
                    wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &wgpu::vertex_attr_array![0 => Float32x3],
                    },
                    wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
                        step_mode: wgpu::VertexStepMode::Instance,
                        attributes: &wgpu::vertex_attr_array![1 => Float32x3, 2 => Float32x4],
                    },
                ],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        let sphere_vertices_glm = geometry::generate_sphere(1.0, 10, 10);
        let sphere_vertices: Vec<[f32; 3]> = sphere_vertices_glm
            .iter()
            .map(|v| [v.x, v.y, v.z])
            .collect();
        let sphere_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Sphere Vertex Buffer"),
            contents: bytemuck::cast_slice(&sphere_vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let num_sphere_vertices = sphere_vertices.len() as u32;

        println!("\nGenerating particle set...");
        let particles = physics::generate_particles(num_particles);
        println!("Done.");

        let instance_data = particles
            .iter()
            .map(|p| InstanceRaw {
                position: [
                    p.position.x as f32,
                    p.position.y as f32,
                    p.position.z as f32,
                ],
                color: [p.color.x, p.color.y, p.color.z, p.color.w],
            })
            .collect::<Vec<_>>();

        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&instance_data),
            usage: wgpu::BufferUsages::VERTEX,
        });

        Self {
            window,
            surface,
            device,
            queue,
            config,
            size,
            camera,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            render_pipeline,
            sphere_vertex_buffer,
            num_sphere_vertices,
            instance_buffer,
            num_instances: particles.len() as u32,
            depth_view,
        }
    }

    pub fn window(&self) -> &Window {
        self.window
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            self.depth_view =
                texture::create_depth_texture(&self.device, &self.config, "depth_texture");
        }
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                event: key_event, ..
            } if key_event.state == ElementState::Pressed => {
                if key_event.logical_key == Key::Named(NamedKey::Escape) {
                    return true;
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let y_offset = match delta {
                    MouseScrollDelta::LineDelta(_, y) => *y as f64,
                    MouseScrollDelta::PixelDelta(pos) => pos.y,
                };
                self.camera.lock().unwrap().process_scroll(y_offset);
            }
            WindowEvent::MouseInput { button, state, .. } => {
                self.camera
                    .lock()
                    .unwrap()
                    .process_mouse_button(*button, *state);
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.camera
                    .lock()
                    .unwrap()
                    .process_mouse_move(position.x, position.y);
            }
            _ => {}
        }
        false
    }

    fn update(&mut self) {
        let projection = glm::perspective_zo(
            self.size.width as f32 / self.size.height as f32,
            glm::radians(&glm::vec1(45.0))[0],
            0.1,
            100.0,
        );
        self.camera_uniform
            .update_view_proj(&self.camera.lock().unwrap(), &projection);
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
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
                            r: 0.05,
                            g: 0.05,
                            b: 0.05,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.sphere_vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
            render_pass.draw(0..self.num_sphere_vertices, 0..self.num_instances);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

fn get_quantum_number(prompt: &str, default: i32) -> i32 {
    loop {
        print!("{} (default: {}): ", prompt, default);
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");

        let trimmed = input.trim();
        if trimmed.is_empty() {
            return default;
        }

        match trimmed.parse::<i32>() {
            Ok(num) => return num,
            Err(_) => {
                println!("Invalid input. Please enter an integer or press Enter for default.")
            }
        }
    }
}

fn get_particle_count() -> usize {
    loop {
        println!("\nSelect particle count:");
        println!("  1. Low    (10,000)");
        println!("  2. Default (100,000)");
        println!("  3. High   (500,000)");
        println!("  4. Custom");
        print!("Enter choice (default: 2): ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");
        let trimmed = input.trim();

        let choice = if trimmed.is_empty() {
            2
        } else {
            match trimmed.parse::<u32>() {
                Ok(c) => c,
                Err(_) => {
                    println!("\nInvalid input. Please enter a number from 1 to 4.");
                    continue;
                }
            }
        };

        match choice {
            1 => return 10_000,
            2 => return 100_000,
            3 => return 500_000,
            4 => loop {
                print!("Enter custom particle count: ");
                io::stdout().flush().unwrap();
                let mut custom_input = String::new();
                io::stdin()
                    .read_line(&mut custom_input)
                    .expect("Failed to read line");
                match custom_input.trim().parse::<usize>() {
                    Ok(num) if num > 0 => return num,
                    _ => println!("Invalid input. Please enter a positive number."),
                }
            },
            _ => {
                println!("\nInvalid choice. Please enter a number from 1 to 4.");
                continue;
            }
        }
    }
}

pub fn main() {
    env_logger::init();
    println!("Enter initial quantum numbers for the simulation.");
    let (n, l, m) = loop {
        let n = get_quantum_number("Principal quantum number (n)", 2);
        let l = get_quantum_number("Azimuthal quantum number (l)", 1);
        let m = get_quantum_number("Magnetic quantum number (m)", 0);

        if n <= 0 {
            println!("\nError: Principal quantum number (n) must be positive.");
            continue;
        }
        if l < 0 || l >= n {
            println!("\nError: Azimuthal quantum number (l) must be in the range [0, n-1].");
            continue;
        }
        if m.abs() > l {
            println!("\nError: Magnetic quantum number (m) must be in the range [-l, l].");
            continue;
        }

        break (n, l, m);
    };

    *physics::N.lock().unwrap() = n;
    *physics::L.lock().unwrap() = l;
    *physics::M.lock().unwrap() = m;

    let num_particles = get_particle_count();

    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new()
        .with_title("Atom Simulator")
        .with_inner_size(winit::dpi::LogicalSize::new(1280, 720))
        .build(&event_loop)
        .unwrap();

    let mut state = pollster::block_on(State::new(&window, num_particles));

    event_loop
        .run(move |event, elwt| match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == state.window().id() => {
                if !state.input(event) {
                    match event {
                        WindowEvent::CloseRequested => {
                            elwt.exit();
                        }
                        WindowEvent::Resized(physical_size) => {
                            state.resize(*physical_size);
                        }
                        WindowEvent::RedrawRequested => {
                            state.update();
                            match state.render() {
                                Ok(_) => {}
                                Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                                Err(wgpu::SurfaceError::OutOfMemory) => elwt.exit(),
                                Err(e) => eprintln!("Error: {:?}", e),
                            }
                        }
                        _ => {}
                    }
                } else {
                    elwt.exit();
                }
            }
            Event::AboutToWait => {
                state.window().request_redraw();
            }
            _ => {}
        })
        .unwrap();
}
