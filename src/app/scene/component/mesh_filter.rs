use imgui::Ui;
use serde::{Deserialize, Serialize};
use wgpu::{util::DeviceExt, Device, RenderBundle};

use crate::{
    app::{
        assets::AssetHandle,
        renderer::{
            model::{Material, Mesh},
            pipeline::Pipeline,
        },
        scene::SceneObject,
        Texture,
    },
    gui::ui,
};

use super::{ComponentIdentifier, ComponentPacker, Transform, TransformStack};

#[derive(Serialize, Deserialize)]
pub struct MeshFilter {
    #[serde(skip)] // until asset handle implenetation
    mesh: Option<AssetHandle<Mesh>>,
    #[serde(skip)] // until asset handle implenetation
    material: Option<AssetHandle<Material>>,

    #[serde(skip)]
    object: Option<SceneObject>,

    #[serde(skip)]
    instance_buffer: Option<wgpu::Buffer>,
}

impl PartialEq for MeshFilter {
    #[allow(unused_variables)]
    fn eq(&self, other: &Self) -> bool {
        false
    }
}

impl MeshFilter {
    pub const IDENT: ComponentIdentifier = "mesh_filter";

    pub fn new() -> MeshFilter {
        MeshFilter {
            mesh: None,
            material: None,
            object: None,
            instance_buffer: None,
        }
    }
    #[allow(dead_code)]
    pub fn with_mesh(mesh: AssetHandle<Mesh>) -> MeshFilter {
        MeshFilter {
            mesh: Some(mesh),
            material: None,
            object: None,
            instance_buffer: None,
        }
    }
    pub fn with_material(mesh: AssetHandle<Mesh>, material: AssetHandle<Material>) -> MeshFilter {
        MeshFilter {
            mesh: Some(mesh),
            material: Some(material),
            object: None,
            instance_buffer: None,
        }
    }

    fn create_default_instance_buffer(device: &Device) -> wgpu::Buffer {
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&[Transform::new().to_raw()]),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        })
    }

    pub(super) fn attach(&mut self, object: SceneObject) {
        self.object = Some(object);
    }

    pub(super) fn gui(&mut self, ui: &Ui) {
        let open = ui
            .tree_node_config("mesh_filter_gui_tree_node")
            .default_open(true)
            .label::<String, String>("Mesh Filter".to_string())
            .framed(true)
            .push();
        if open.is_some() {
            ui::text_label(ui, "Mesh:");
            ui.text(
                self.mesh
                    .as_ref()
                    .map_or("None".to_string(), |f| f.location.to_ident()),
            );
            ui::text_label(ui, "Material:");
            ui.text(
                self.material
                    .as_ref()
                    .map_or("None".to_string(), |f| f.location.to_ident()),
            );
        }
    }
}

impl Default for MeshFilter {
    fn default() -> Self {
        Self::new()
    }
}

impl ComponentPacker for MeshFilter {
    fn pack(self) -> super::Component {
        super::Component::MeshFilter(self)
    }
}

impl MeshFilter {
    pub fn render(
        &mut self,
        pipeline: &Pipeline,
        device: &wgpu::Device,
        bind_groups: &[&wgpu::BindGroup],
        queue: &wgpu::Queue,
        transform_stack: &mut TransformStack,
    ) -> Option<RenderBundle> {
        //TODO: Track changes and save recorded bundle
        if let Some(mesh) = &self.mesh {
            let mut encoder =
                device.create_render_bundle_encoder(&wgpu::RenderBundleEncoderDescriptor {
                    label: Some(&format!("Mesh Filter Encoder: {}", pipeline.name)),
                    color_formats: &pipeline.color_formats,
                    depth_stencil: Some(wgpu::RenderBundleDepthStencil {
                        format: Texture::DEPTH_FORMAT,
                        depth_read_only: false,
                        stencil_read_only: false,
                    }),
                    sample_count: 1,
                    multiview: None,
                });

            encoder.set_pipeline(&pipeline.pipeline);

            for (i, group) in bind_groups.iter().enumerate() {
                // +1 because 0 is DiffuseBindGroup
                encoder.set_bind_group(i as u32 + 1, &group, &[]);
            }

            if self.instance_buffer.is_none() {
                self.instance_buffer = Some(Self::create_default_instance_buffer(device));
            }
            let instance_buffer = self
                .instance_buffer
                .as_ref()
                .expect("Should have been build before!");
            queue.write_buffer(
                instance_buffer,
                0,
                bytemuck::cast_slice(&[transform_stack.eval()]), //TODO: Cash hash or something to prevent reupload every frame
            );
            encoder.set_vertex_buffer(1, instance_buffer.slice(..));

            // encoder.draw_mesh_instanced(
            //     device,
            //     &pipeline.texture_bind_group_layout,
            //     mesh.clone(),
            //     self.material.as_mut().unwrap(),
            //     0..1,
            // )

            let m = mesh.asset.borrow();

            encoder.set_vertex_buffer(0, m.vertex_buffer.slice(..));
            encoder.set_index_buffer(m.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

            let material = self.material.as_ref().unwrap().asset.borrow();
            let mut diffuse = material.diffuse_texture.asset.borrow_mut();

            encoder.set_bind_group(
                0,
                diffuse.bind_group(device, &pipeline.texture_bind_group_layout),
                &[],
            );
            encoder.draw_indexed(0..mesh.asset.borrow().num_elements, 0, 0..1);

            Some(encoder.finish(&wgpu::RenderBundleDescriptor {
                label: Some(&format!(
                    "Render Bundle for: {}",
                    (self.object.as_ref().expect("Why not attached")).name()
                )),
            }))
        } else {
            None
        }
    }
}
