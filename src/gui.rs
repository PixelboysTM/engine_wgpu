use std::{
    borrow::BorrowMut,
    cell::{Ref, RefCell},
    ops::{Deref, DerefMut},
    rc::Rc,
    time::Duration,
};

use imgui::{FontSource, Io, Ui};
use imgui_wgpu::{Renderer, RendererConfig};
use imgui_winit_support::winit::window::Window;
use wgpu::RenderPass;
use winit::event::Event;

use crate::app::Texture;

pub fn init_gui(
    window: &Window,
    format: &wgpu::TextureFormat,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
) -> (Gui, GuiPlatform) {
    let hidpi_factor = window.scale_factor();

    let mut imgui = imgui::Context::create();
    let mut platform = imgui_winit_support::WinitPlatform::init(&mut imgui);
    platform.attach_window(
        imgui.io_mut(),
        window,
        imgui_winit_support::HiDpiMode::Default,
    );
    imgui.set_ini_filename(None);

    let font_size = (13.0 * hidpi_factor) as f32;
    imgui.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;

    imgui.fonts().add_font(&[FontSource::DefaultFontData {
        config: Some(imgui::FontConfig {
            oversample_h: 1,
            pixel_snap_h: true,
            size_pixels: font_size,
            ..Default::default()
        }),
    }]);

    let renderer_config = RendererConfig {
        texture_format: *format,
        depth_format: Some(Texture::DEPTH_FORMAT),
        ..Default::default()
    };

    let mut renderer = Renderer::new(&mut imgui, device, queue, renderer_config);

    (
        Gui {
            context: imgui,
            renderer,
        },
        GuiPlatform(platform),
    )
}

pub struct Gui {
    context: imgui::Context,
    renderer: Renderer,
}

pub struct GuiPlatform(imgui_winit_support::WinitPlatform);

impl GuiPlatform {
    pub fn end_frame(&mut self, ui: &mut Ui, window: &Window) {
        self.0.prepare_render(ui, window);
    }
    pub fn handle_event<T>(&mut self, io: &mut Io, window: &Window, event: &Event<T>) {
        self.0.handle_event(io, window, event);
    }
}

impl Gui {
    pub fn update(
        &mut self,
        dt: Duration,
        window: &Window,
        gui_platform: &mut GuiPlatform,
    ) -> &mut Ui {
        self.context.io_mut().update_delta_time(dt);
        gui_platform
            .0
            .prepare_frame(self.context.io_mut(), window)
            .expect("Failed to prepare frame.");
        let ui = self.context.frame();
        ui
    }

    pub fn render<'r>(
        &'r mut self,
        queue: &wgpu::Queue,
        device: &wgpu::Device,
        rpass: &mut RenderPass<'r>,
    ) {
        self.renderer
            .render(self.context.render(), queue, device, rpass)
            .expect("Rendering failed");
    }

    pub fn io_mut(&mut self) -> &mut Io {
        self.context.io_mut()
    }
}
