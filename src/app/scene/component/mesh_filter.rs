use imgui::Ui;
use serde::{Deserialize, Serialize};
use wgpu::RenderBundle;

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

use super::{ComponentIdentifier, ComponentPacker, TransformStack};

#[derive(Serialize, Deserialize)]
pub struct MeshFilter {
    #[serde(skip)] // until asset handle implenetation
    mesh: Option<AssetHandle<Mesh>>,
    #[serde(skip)] // until asset handle implenetation
    material: Option<Material>,

    #[serde(skip)]
    object: Option<SceneObject>,
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
        }
    }
    #[allow(dead_code)]
    pub fn with_mesh(mesh: AssetHandle<Mesh>) -> MeshFilter {
        MeshFilter {
            mesh: Some(mesh),
            material: None,
            object: None,
        }
    }
    pub fn with_material(mesh: AssetHandle<Mesh>, material: Material) -> MeshFilter {
        MeshFilter {
            mesh: Some(mesh),
            material: Some(material),
            object: None,
        }
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
                    .map_or("None".to_string(), |f| f.asset.borrow().name.clone()),
            );
            ui::text_label(ui, "Material:");
            ui.text(self.material.as_ref().map_or("None", |f| &f.name));
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
        instance_buffer: &wgpu::Buffer,
        queue: &wgpu::Queue,
        transform_stack: &mut TransformStack,
    ) -> Option<RenderBundle> {
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

            queue.write_buffer(
                instance_buffer,
                0,
                bytemuck::cast_slice(&[transform_stack.eval()]),
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
            encoder.set_bind_group(
                0,
                self.material
                    .as_mut()
                    .unwrap()
                    .diffuse_texture
                    .bind_group(device, &pipeline.texture_bind_group_layout),
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
