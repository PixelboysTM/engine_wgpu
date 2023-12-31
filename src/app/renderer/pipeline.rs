use wgpu::{BindGroupLayout, ColorTargetState, Device, Queue, RenderBundle, TextureFormat};

use crate::app::scene::{
    component::{Component, MeshFilter, TransformRaw, TransformStack},
    Scene, SceneObject,
};

use super::{
    mesh::{MeshVertex, Vertex},
    texture::Texture,
};

pub struct Pipeline {
    pub(crate) pipeline: wgpu::RenderPipeline,
    pub(crate) name: String,
    pub(crate) color_formats: Vec<Option<TextureFormat>>,
    pub(crate) texture_bind_group_layout: wgpu::BindGroupLayout,
}

impl Pipeline {
    pub fn new(
        device: &Device,
        name: &str,
        shader_source: &str,
        color_targets: &[ColorTargetState],
        bind_group_layouts: &[&BindGroupLayout],
    ) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(&format!("Shader: {name}")),
            source: wgpu::ShaderSource::Wgsl(shader_source.into()),
        });

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("Texture bind group"),
            });

        let mut bind_group_layouts = bind_group_layouts.to_vec();
        bind_group_layouts.insert(0, &texture_bind_group_layout);

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some(&format!("Render Pipeline Layout: {name}")),
                bind_group_layouts: &bind_group_layouts,
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(&format!("Render Pipeline: {name}")),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[MeshVertex::desc(), TransformRaw::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &color_targets
                    .iter()
                    .map(|c| Some(c.clone()))
                    .collect::<Vec<_>>(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: Texture::DEPTH_FORMAT,
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

        Self {
            pipeline: render_pipeline,
            name: name.to_string(),
            color_formats: color_targets.iter().map(|c| Some(c.format)).collect(),
            texture_bind_group_layout,
        }
    }

    pub(super) fn render_scene(
        &mut self,
        scene: &Scene,
        device: &Device,
        queue: &Queue,
        bind_groups: &[&wgpu::BindGroup],
    ) -> Vec<RenderBundle> {
        let mut transform_stack = TransformStack::new();
        self.render_scene_obj(
            scene.root(),
            device,
            queue,
            bind_groups,
            &mut transform_stack,
        )
    }

    fn render_scene_obj(
        &self,
        obj: SceneObject,
        device: &Device,
        queue: &Queue,
        bind_groups: &[&wgpu::BindGroup],
        transform_stack: &mut TransformStack,
    ) -> Vec<RenderBundle> {
        transform_stack.push(obj.get_transform().to_raw());

        let mut bundles = vec![];
        let mf = obj.get_component(MeshFilter::IDENT);
        if let Some(mesh_filter) = mf {
            let mfc: &mut Component = &mut mesh_filter.get().borrow_mut();
            match mfc {
                Component::MeshFilter(filter) => {
                    let b = filter.render(self, device, bind_groups, queue, transform_stack);
                    if let Some(b) = b {
                        bundles.push(b);
                    }
                }
            }
        }

        for child in obj.children() {
            bundles.append(&mut self.render_scene_obj(
                child,
                device,
                queue,
                bind_groups,
                transform_stack,
            ));
        }

        transform_stack.pop();

        bundles
    }
}
