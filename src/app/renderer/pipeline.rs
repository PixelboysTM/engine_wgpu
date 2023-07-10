use std::ops::Range;

use wgpu::{
    BindGroupLayout, BufferSlice, ColorTargetState, Device, Queue, RenderBundle, TextureFormat,
};

use super::{
    mesh::{MeshVertex, Vertex},
    model::{DrawModel, Material, Mesh},
    texture::Texture,
    InstanceRaw,
};

pub struct Pipeline {
    pipeline: wgpu::RenderPipeline,
    name: String,
    color_formats: Vec<Option<TextureFormat>>,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    diffuse_texture: Texture,
}

impl Pipeline {
    pub fn new(
        device: &Device,
        queue: &Queue,
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
                buffers: &[MeshVertex::desc(), InstanceRaw::desc()],
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

        let diffuse_bytes = include_bytes!("../textures/happy-tree.png");
        let diffuse_texture =
            Texture::from_bytes(device, queue, diffuse_bytes, "Happy Tree").unwrap();

        Self {
            pipeline: render_pipeline,
            name: name.to_string(),
            color_formats: color_targets.iter().map(|c| Some(c.format)).collect(),
            diffuse_texture,
            texture_bind_group_layout,
        }
    }

    pub(super) fn draw(
        &mut self,
        device: &Device,
        bind_groups: &[&wgpu::BindGroup],
        vertex_buffers: &[BufferSlice],
        mesh: &Mesh,
        material: &mut Material,
        instances: Range<u32>,
    ) -> RenderBundle {
        let mut encoder =
            device.create_render_bundle_encoder(&wgpu::RenderBundleEncoderDescriptor {
                label: Some(&format!("Render Bundle Encoder: {}", self.name)),
                color_formats: &self.color_formats,
                depth_stencil: Some(wgpu::RenderBundleDepthStencil {
                    format: Texture::DEPTH_FORMAT,
                    depth_read_only: false,
                    stencil_read_only: false,
                }),
                sample_count: 1,
                multiview: None,
            });

        encoder.set_pipeline(&self.pipeline);
        // encoder.set_bind_group(
        //     0,
        //     self.diffuse_texture
        //         .bind_group(device, &self.texture_bind_group_layout),
        //     &[],
        // );
        for (i, group) in bind_groups.iter().enumerate() {
            // +1 because 0 is DiffuseBindGroup
            encoder.set_bind_group(i as u32 + 1, &group, &[]);
        }
        for (i, slice) in vertex_buffers.iter().enumerate() {
            // +1 becaise 0 is Vertex data from model
            encoder.set_vertex_buffer(i as u32 + 1, *slice);
        }
        encoder.draw_mesh_instanced(
            device,
            &self.texture_bind_group_layout,
            mesh,
            material,
            instances,
        );

        encoder.finish(&wgpu::RenderBundleDescriptor {
            label: Some(&format!("Render Bundle: {}", self.name)),
        })
    }
}
