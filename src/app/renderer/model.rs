use std::io::{BufReader, Cursor};

use anyhow::Ok;
use wgpu::util::DeviceExt;

use crate::app::assets::{AssetDatabase, AssetHandle, AssetLocation};

use super::{mesh::MeshVertex, texture::Texture};

pub struct Model {
    pub meshes: Vec<AssetHandle<Mesh>>,
    pub materials: Vec<AssetHandle<Material>>,
}

pub struct Material {
    pub name: String,
    pub diffuse_texture: AssetHandle<Texture>,
}

pub struct Mesh {
    pub name: String,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_elements: u32,
    pub material: usize,
}

// pub trait DrawModel<'a> {
//     fn draw_model(
//         &mut self,
//         device: &wgpu::Device,
//         bind_group_layout: &wgpu::BindGroupLayout,
//         mesh: AssetHandle<Mesh>,
//         material: &'a mut Material,
//     );
//     fn draw_mesh_instanced(
//         &mut self,
//         device: &wgpu::Device,
//         bind_group_layout: &wgpu::BindGroupLayout,
//         mesh: AssetHandle<Mesh>,
//         material: &'a mut Material,
//         instances: Range<u32>,
//     );
// }

// impl<'a, 'b> DrawModel<'b> for wgpu::RenderBundleEncoder<'a>
// where
//     'b: 'a,
// {
//     fn draw_model(
//         &mut self,
//         device: &wgpu::Device,
//         bind_group_layout: &wgpu::BindGroupLayout,
//         mesh: AssetHandle<Mesh>,
//         material: &'a mut Material,
//     ) {
//         self.draw_mesh_instanced(device, bind_group_layout, mesh, material, 0..1);
//     }

//     fn draw_mesh_instanced(
//         &mut self,
//         device: &wgpu::Device,
//         bind_group_layout: &wgpu::BindGroupLayout,
//         mesh: AssetHandle<Mesh>,
//         material: &'a mut Material,
//         instances: Range<u32>,
//     ) {
//         let mesh = mesh.asset.borrow();
//         self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
//         self.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
//         self.set_bind_group(
//             0,
//             material
//                 .diffuse_texture
//                 .bind_group(device, bind_group_layout),
//             &[],
//         );
//         self.draw_indexed(0..mesh.num_elements, 0, instances);
//     }
// }

pub async fn load_model(
    file_name: &str,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    asset_databse: AssetDatabase,
) -> anyhow::Result<Model> {
    let obj_text = load_string(file_name).await?;
    let obj_cursor = Cursor::new(obj_text);
    let mut obj_reader = BufReader::new(obj_cursor);

    let (models, obj_materials) = tobj::load_obj_buf_async(
        &mut obj_reader,
        &tobj::LoadOptions {
            single_index: true,
            triangulate: true,
            ..Default::default()
        },
        |p| async move {
            let mat_text = load_string(&p).await.unwrap();
            tobj::load_mtl_buf(&mut BufReader::new(Cursor::new(mat_text)))
        },
    )
    .await?;

    let mut materials = Vec::new();
    for m in obj_materials? {
        let tex_name = m.diffuse_texture.unwrap();

        let diffuse_texture = Texture::load_texture(&tex_name, device, queue).await?;

        let tex = asset_databse.load_texture(
            AssetLocation::Resource {
                path: tex_name,
                in_file_ident: None,
            },
            diffuse_texture,
        );
        materials.push(asset_databse.load_material(
            AssetLocation::Resource {
                path: file_name.to_string(),
                in_file_ident: Some(m.name.clone()),
            },
            Material {
                diffuse_texture: tex,
                name: m.name,
            },
        ));
    }

    let meshes = models
        .into_iter()
        .map(|m| {
            let vertices = (0..m.mesh.positions.len() / 3)
                .map(|i| MeshVertex {
                    position: [
                        m.mesh.positions[i * 3],
                        m.mesh.positions[i * 3 + 1],
                        m.mesh.positions[i * 3 + 2],
                    ],
                    tex_coords: [m.mesh.texcoords[i * 2], m.mesh.texcoords[i * 2 + 1]],
                    normal: [
                        m.mesh.normals[i * 3],
                        m.mesh.normals[i * 3 + 1],
                        m.mesh.normals[i * 3 + 2],
                    ],
                })
                .collect::<Vec<_>>();

            let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{:?} Vertex Buffer", file_name)),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });
            let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{:?} Index Buffer", file_name)),
                contents: bytemuck::cast_slice(&m.mesh.indices),
                usage: wgpu::BufferUsages::INDEX,
            });

            asset_databse.load_mesh(
                AssetLocation::Resource {
                    path: file_name.to_string(),
                    in_file_ident: Some(m.name),
                },
                Mesh {
                    name: file_name.to_string(),
                    vertex_buffer,
                    index_buffer,
                    num_elements: m.mesh.indices.len() as u32,
                    material: m.mesh.material_id.unwrap_or(0),
                },
            )
        })
        .collect::<Vec<_>>();

    Ok(Model { meshes, materials })
}

pub async fn load_string(file_name: &str) -> anyhow::Result<String> {
    let path = std::path::Path::new(env!("OUT_DIR"))
        .join("res")
        .join(file_name);
    let txt = std::fs::read_to_string(path)?;

    Ok(txt)
}

pub async fn load_binary(file_name: &str) -> anyhow::Result<Vec<u8>> {
    let path = std::path::Path::new(env!("OUT_DIR"))
        .join("res")
        .join(file_name);
    let data = std::fs::read(path)?;

    Ok(data)
}
