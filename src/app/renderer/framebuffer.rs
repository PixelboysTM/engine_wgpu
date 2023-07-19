use crate::app::Texture;

pub struct Framebuffer {
    diffuse: Texture,
    depth: Texture,
    width: u32,
    height: u32,
    diffuse_format: wgpu::TextureFormat,
    name: String,
}

impl Framebuffer {
    pub fn create(
        device: &wgpu::Device,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
        name: &str,
    ) -> Self {
        let diffuse = Texture::create_texture(
            device,
            (width, height),
            format,
            &format!("Framebuffer diffuse: {name}"),
        );
        let depth = Texture::create_texture(
            device,
            (width, height),
            Texture::DEPTH_FORMAT,
            &format!("Framebuffer depth: {name}"),
        );

        Self {
            diffuse,
            depth,
            width,
            height,
            diffuse_format: format,
            name: name.to_string(),
        }
    }

    pub fn diffuse_view(&self) -> &wgpu::TextureView {
        self.diffuse.view()
    }
    pub fn depth_view(&self) -> &wgpu::TextureView {
        self.depth.view()
    }
    #[allow(dead_code)]
    pub fn depth(&self) -> &Texture {
        &self.depth
    }
    pub fn diffuse(&self) -> &Texture {
        &self.diffuse
    }
    #[allow(dead_code)]
    pub fn width(&self) -> u32 {
        self.width
    }
    #[allow(dead_code)]
    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn resize(&mut self, width: u32, height: u32, device: &wgpu::Device) -> bool {
        if (self.width != width || self.height != height) && (width > 0 && height > 0) {
            self.diffuse = Texture::create_texture(
                device,
                (width, height),
                self.diffuse_format,
                &format!("Framebuffer diffuse: {}", self.name),
            );
            self.depth = Texture::create_texture(
                device,
                (width, height),
                Texture::DEPTH_FORMAT,
                &format!("Framebuffer depth: {}", self.name),
            );
            self.width = width;
            self.height = height;
            true
        } else {
            false
        }
    }
}
