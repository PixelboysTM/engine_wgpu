mod renderer;

use std::time::Duration;

use wgpu::util::DeviceExt;
use winit::{
    dpi::PhysicalSize,
    event::{Event, WindowEvent},
    window::Window,
};

pub(crate) use renderer::texture::*;

use self::renderer::{
    model::{load_model, Model},
    Instance, Renderer,
};
use cgmath::prelude::*;

const NUM_INSTANCES_PER_ROW: u32 = 10;

pub struct ApplicationState {
    renderer: Renderer,
    size: winit::dpi::PhysicalSize<u32>,
    window: Window,
    obj_model: Model,
    instances: Vec<Instance>,
    instance_buffer: wgpu::Buffer,
}

impl ApplicationState {
    pub(super) async fn new(window: Window) -> Self {
        let size = window.inner_size();

        let renderer = Renderer::new(&window, size).await;

        let obj_model = load_model("cube.obj", renderer.device(), renderer.queue())
            .await
            .unwrap();

        const SPACE_BETWEEN: f32 = 3.0;
        let instances = (0..NUM_INSTANCES_PER_ROW)
            .flat_map(|z| {
                (0..NUM_INSTANCES_PER_ROW).map(move |x| {
                    let x = SPACE_BETWEEN * (x as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0);
                    let z = SPACE_BETWEEN * (z as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0);

                    let position = cgmath::Vector3 { x, y: 0.0, z };

                    let rotation = if position.is_zero() {
                        cgmath::Quaternion::from_axis_angle(
                            cgmath::Vector3::unit_z(),
                            cgmath::Deg(0.0),
                        )
                    } else {
                        cgmath::Quaternion::from_axis_angle(position.normalize(), cgmath::Deg(45.0))
                    };

                    Instance { position, rotation }
                })
            })
            .collect::<Vec<_>>();

        let instance_data = instances.iter().map(Instance::to_raw).collect::<Vec<_>>();
        let instance_buffer =
            renderer
                .device()
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Instance Buffer"),
                    contents: bytemuck::cast_slice(&instance_data),
                    usage: wgpu::BufferUsages::VERTEX,
                });

        Self {
            renderer,
            size,
            window,
            obj_model,
            instances,
            instance_buffer,
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub(super) fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.renderer.resize(new_size);
        }
    }

    pub(super) fn input(&mut self, event: &WindowEvent) -> bool {
        false
    }

    pub(super) fn event<T>(&mut self, event: &Event<T>) {
        self.renderer.event(&self.window, event);
    }

    pub(super) fn update(&mut self, dt: Duration) {
        self.renderer.update(dt, &self.window);
    }

    pub(super) fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        self.renderer.render(
            &mut self.obj_model,
            self.instance_buffer.slice(..),
            0..self.instances.len() as _,
        )
    }

    pub(super) fn size(&self) -> &PhysicalSize<u32> {
        &self.size
    }
}
