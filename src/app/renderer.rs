mod camera;
mod framebuffer;
pub mod mesh;
pub(crate) mod model;
pub mod pipeline;
pub(crate) mod texture;

use std::time::Duration;

use imgui::TextureId;
use wgpu::{util::DeviceExt, ColorTargetState, Device, Queue};
use winit::{dpi::PhysicalSize, event::Event, window::Window};

#[cfg(feature = "imgui")]
use crate::gui::{init_gui, Gui, GuiPlatform};

use self::{camera::Camera, framebuffer::Framebuffer, pipeline::Pipeline};

use super::scene::{component::Transform, Scene};

pub struct Renderer {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    render_pipeline: Pipeline,
    camera: Camera,
    framebuffer: Framebuffer,
    framebuffer_gui_id: TextureId,
    gui_viewport_size: [u32; 2],
    depth_texture: texture::Texture,
    instance_buffer: wgpu::Buffer,
    #[cfg(feature = "imgui")]
    gui: Gui,
    gui_platform: GuiPlatform,
}

impl Renderer {
    pub async fn new(window: &Window, size: PhysicalSize<u32>) -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            dx12_shader_compiler: Default::default(),
        });

        let surface = unsafe { instance.create_surface(&window) }.unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                    label: Some("Device, Queue"),
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
        };
        surface.configure(&device, &config);

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

        let camera = Camera::new(
            &device,
            (0.0, 5.0, 10.0),
            (0.0, 0.0, 0.0),
            cgmath::Vector3::unit_y(),
            config.width as f32 / config.height as f32,
            45.0,
            0.1,
            100.0,
            &camera_bind_group_layout,
        );

        let render_pipeline = Pipeline::new(
            &device,
            "Main Renderer",
            include_str!("shaders/shader.wgsl"),
            &[ColorTargetState {
                format: config.format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            }],
            &[&camera_bind_group_layout],
        );

        let depth_texture =
            texture::Texture::create_depth_texture(&device, &config, "depth_texture");
        let framebuffer = Framebuffer::create(&device, 800, 600, config.format, "Main");

        let instance_data = [Transform::new().to_raw()];
        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&instance_data),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        #[cfg(feature = "imgui")]
        let (mut gui, gui_platform) = init_gui(window, &config.format, &device, &queue);
        let framebuffer_gui_id = gui.insert_texture(&device, framebuffer.diffuse());

        Self {
            surface,
            device,
            queue,
            config,

            render_pipeline,
            camera,
            framebuffer,
            framebuffer_gui_id,
            depth_texture,
            instance_buffer,
            #[cfg(feature = "imgui")]
            gui,
            #[cfg(feature = "imgui")]
            gui_platform,
            gui_viewport_size: [0; 2],
        }
    }

    pub(super) fn resize(&mut self, new_size: PhysicalSize<u32>) {
        self.config.width = new_size.width;
        self.config.height = new_size.height;
        self.surface.configure(&self.device, &self.config);
        self.depth_texture =
            texture::Texture::create_depth_texture(&self.device, &self.config, "depth_texture");
        // self.camera.update_aspect(
        //     &self.queue,
        //     self.config.width as f32 / self.config.height as f32,
        // );
    }

    pub(super) fn update(&mut self, dt: Duration, window: &Window, scene: &Scene) {
        //GUI
        {
            let ui = self.gui.update(dt, window, &mut self.gui_platform);

            let mut open: bool = true;
            ui.dockspace_over_main_viewport();
            ui.show_demo_window(&mut open);

            //Viewport
            {
                let _token = ui.push_style_var(imgui::StyleVar::WindowPadding([0.0, 0.0]));
                if let Some(_) = ui.window("Viewport").begin() {
                    let size = ui.content_region_avail();

                    imgui::Image::new(self.framebuffer_gui_id, size).build(ui);

                    self.gui_viewport_size = [size[0] as u32, size[1] as u32];
                }
            }

            scene.gui(ui);

            self.gui_platform.end_frame(ui, window);
        }
        //GUI
    }

    pub(super) fn event<T>(&mut self, window: &Window, event: &Event<T>) {
        self.gui_platform
            .handle_event(self.gui.io_mut(), window, event);
    }

    pub(super) fn render(&mut self, scene: &Scene) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        //Scene
        {
            if self.framebuffer.resize(
                self.gui_viewport_size[0],
                self.gui_viewport_size[1],
                &self.device,
            ) {
                self.camera.update_aspect(
                    &self.queue,
                    self.gui_viewport_size[0] as f32 / self.gui_viewport_size[1] as f32,
                );
                self.gui.update_texture(
                    self.framebuffer_gui_id,
                    self.framebuffer.diffuse(),
                    &self.device,
                );
            }

            let bundles = self.render_pipeline.render_scene(
                scene,
                &self.device,
                &self.queue,
                &self.instance_buffer,
                &[&self.camera.bind_group()],
            );

            let mut scene_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: self.framebuffer.diffuse_view(),
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: self.framebuffer.depth_view(),
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            scene_pass.execute_bundles(&bundles);
        }

        //Main Window
        {
            // let bundle = self.render_pipeline.draw(
            //     &self.device,
            //     &[self.camera.bind_group()],
            //     &[],
            //     &object.meshes[0],
            //     &mut object.materials[object.meshes[0].material],
            //     0..1,
            // );

            let mut main_window_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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
                    view: self.depth_texture.view(),
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            //GUI
            {
                self.gui
                    .render(&self.queue, &self.device, &mut main_window_pass);
            }
            //GUI
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    pub(super) fn device(&self) -> &Device {
        &self.device
    }

    pub(super) fn queue(&self) -> &Queue {
        &self.queue
    }
}

// pub(super) struct Instance {
//     pub(super) position: cgmath::Vector3<f32>,
//     pub(super) rotation: cgmath::Quaternion<f32>,
// }

// impl Instance {
//     pub(super) fn to_raw(&self) -> TranslationRaw {
//         TranslationRaw {
//             model: (cgmath::Matrix4::from_translation(self.position)
//                 * cgmath::Matrix4::from(self.rotation))
//             .into(),
//         }
//     }
// }
