use cgmath::{Matrix4, SquareMatrix};
use imgui::{Drag, Ui};
use serde::{Deserialize, Serialize};

use crate::{app::renderer::mesh::Vertex, gui::ui};

#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub struct Transform {
    position: cgmath::Vector3<f32>,
    rotation: cgmath::Vector3<f32>,
    scale: cgmath::Vector3<f32>,
}

impl Transform {
    pub fn new() -> Transform {
        Transform {
            position: cgmath::vec3(0.0, 0.0, 0.0),
            rotation: cgmath::vec3(0.0, 0.0, 0.0),
            scale: cgmath::vec3(1.0, 1.0, 1.0),
        }
    }
}

impl Transform {
    pub(super) fn gui(&mut self, ui: &Ui) {
        let open = ui
            .tree_node_config("transform_gui_tree_node")
            .default_open(true)
            .label::<String, String>("Transform".to_string())
            .framed(true)
            .push();
        if open.is_some() {
            ui::text_label(ui, "Position:");
            let mut pos: [f32; 3] = self.position.into();
            if Drag::new("##transform_input_pos")
                .speed(0.1)
                .build_array(ui, &mut pos)
            {
                self.position = pos.into();
            }

            ui::text_label(ui, "Rotation:");
            let mut rot: [f32; 3] = self.rotation.into();
            if Drag::new("##transform_input_rot")
                .speed(0.1)
                .build_array(ui, &mut rot)
            {
                self.rotation = rot.into();
            }

            ui::text_label(ui, "Size:");
            let mut size: [f32; 3] = self.scale.into();
            if Drag::new("##transform_input_size")
                .speed(0.1)
                .build_array(ui, &mut size)
            {
                self.scale = size.into();
            }
        }
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::new()
    }
}

impl Transform {
    pub(crate) fn to_raw(&self) -> TransformRaw {
        TransformRaw {
            model: (cgmath::Matrix4::from_nonuniform_scale(
                self.scale.x,
                self.scale.y,
                self.scale.z,
            ) * cgmath::Matrix4::from(cgmath::Quaternion::from(cgmath::Euler::new(
                cgmath::Deg(self.rotation.x),
                cgmath::Deg(self.rotation.y),
                cgmath::Deg(self.rotation.z),
            ))) * cgmath::Matrix4::from_translation(self.position))
            .into(),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TransformRaw {
    pub(crate) model: [[f32; 4]; 4],
}

impl Vertex for TransformRaw {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<TransformRaw>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 10,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 11,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 12,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 13,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

pub struct TransformStack {
    stack: Vec<Matrix4<f32>>,
}

impl TransformStack {
    pub fn new() -> Self {
        Self { stack: vec![] }
    }
    pub fn push(&mut self, transform: TransformRaw) {
        self.stack.push(Matrix4::from(transform.model));
    }
    pub fn pop(&mut self) -> Option<TransformRaw> {
        self.stack.pop().map(|f| TransformRaw { model: f.into() })
    }
    pub fn eval(&self) -> TransformRaw {
        let mat = self
            .stack
            .iter()
            .fold(Matrix4::identity(), |acc, mat| acc * mat);
        TransformRaw { model: mat.into() }
    }
}
