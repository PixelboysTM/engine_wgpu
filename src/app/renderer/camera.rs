use wgpu::{util::DeviceExt, BindGroup, Device};

pub struct Camera {
    eye: cgmath::Point3<f32>,
    target: cgmath::Point3<f32>,
    up: cgmath::Vector3<f32>,
    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,

    uniform: CameraUniform,
    buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

impl Camera {
    pub fn new<P: Into<cgmath::Point3<f32>>, V: Into<cgmath::Vector3<f32>>>(
        device: &Device,
        eye: P,
        target: P,
        up: V,
        aspect: f32,
        fovy: f32,
        znear: f32,
        zfar: f32,
        camera_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Camera {
        let eye = eye.into();
        let target = target.into();
        let up = up.into();

        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(Camera::build_view_projection_matrix(
            eye, target, up, aspect, fovy, znear, zfar,
        ));

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });
        Self {
            aspect,
            bind_group: camera_bind_group,
            buffer: camera_buffer,
            uniform: camera_uniform,
            eye,
            target,
            fovy,
            zfar,
            znear,
            up,
        }
    }

    pub fn bind_group(&self) -> &BindGroup {
        &self.bind_group
    }
    pub fn update_aspect(&mut self, queue: &wgpu::Queue, aspect: f32) {
        self.aspect = aspect;
        self.uniform
            .update_view_proj(Camera::build_view_projection_matrix(
                self.eye,
                self.target,
                self.up,
                self.aspect,
                self.fovy,
                self.znear,
                self.zfar,
            ));
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[self.uniform]));
    }
}

impl Camera {
    fn build_view_projection_matrix(
        eye: cgmath::Point3<f32>,
        target: cgmath::Point3<f32>,
        up: cgmath::Vector3<f32>,
        aspect: f32,
        fovy: f32,
        znear: f32,
        zfar: f32,
    ) -> cgmath::Matrix4<f32> {
        let view = cgmath::Matrix4::look_at_rh(eye, target, up);
        let proj = cgmath::perspective(cgmath::Deg(fovy), aspect, znear, zfar);

        OPENGL_TO_WGPU_MATRIX * proj * view
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniform {
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    fn new() -> Self {
        use cgmath::SquareMatrix;
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }

    fn update_view_proj(&mut self, camera_matrix: cgmath::Matrix4<f32>) {
        self.view_proj = camera_matrix.into();
    }
}

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);
