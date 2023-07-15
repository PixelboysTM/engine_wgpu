use std::{path::PathBuf, time::Duration};

use imgui::{ConfigFlags, Direction, FontSource, Io, StyleColor, Ui};
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
    imgui.set_ini_filename(Some(PathBuf::from("gui_layout.ini")));

    let font_size = (19.0 * hidpi_factor) as f32;
    imgui.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;

    imgui.fonts().add_font(&[FontSource::TtfData {
        data: include_bytes!("gui/RobotoMono-Regular.ttf"),
        size_pixels: font_size,
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

    let renderer = Renderer::new(&mut imgui, device, queue, renderer_config);

    set_dark_theme(&mut imgui);

    imgui.io_mut().config_flags |= ConfigFlags::DOCKING_ENABLE | ConfigFlags::VIEWPORTS_ENABLE;
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

fn set_dark_theme(context: &mut imgui::Context) {
    let styles = context.style_mut();

    styles.window_padding = [8.0, 8.0];
    styles.frame_padding = [5.0, 2.0];
    styles.cell_padding = [6.0, 6.0];
    styles.item_spacing = [6.0, 6.0];
    styles.item_inner_spacing = [6.0, 6.0];
    styles.touch_extra_padding = [0.0, 0.0];
    styles.indent_spacing = 25.0;
    styles.scrollbar_size = 15.0;
    styles.grab_min_size = 10.0;
    styles.window_border_size = 1.0;
    styles.child_border_size = 1.0;
    styles.popup_border_size = 1.0;
    styles.frame_border_size = 1.0;
    styles.tab_border_size = 1.0;
    styles.window_rounding = 7.0;
    styles.child_rounding = 4.0;
    styles.frame_rounding = 3.0;
    styles.popup_rounding = 4.0;
    styles.scrollbar_rounding = 9.0;
    styles.grab_rounding = 3.0;
    styles.log_slider_deadzone = 4.0;
    styles.tab_rounding = 4.0;

    styles.window_title_align = [0.5, 0.5];
    styles.window_menu_button_position = Direction::None;
    styles.color_button_position = Direction::Left;
    styles.button_text_align = [0.5, 0.5];
    styles.circle_tesselation_max_error = 0.1;

    styles.colors[StyleColor::Text as usize] = [1.00, 1.00, 1.00, 1.00];
    styles.colors[StyleColor::TextDisabled as usize] = [0.50, 0.50, 0.50, 1.00];
    styles.colors[StyleColor::WindowBg as usize] = [0.10, 0.10, 0.10, 1.00];
    styles.colors[StyleColor::ChildBg as usize] = [0.00, 0.00, 0.00, 0.00];
    styles.colors[StyleColor::PopupBg as usize] = [0.19, 0.19, 0.19, 1.00];
    styles.colors[StyleColor::Border as usize] = [0.19, 0.19, 0.19, 0.29];
    styles.colors[StyleColor::BorderShadow as usize] = [0.00, 0.00, 0.00, 0.24];
    styles.colors[StyleColor::FrameBg as usize] = [0.05, 0.05, 0.05, 0.54];
    styles.colors[StyleColor::FrameBgHovered as usize] = [0.19, 0.19, 0.19, 0.54];
    styles.colors[StyleColor::FrameBgActive as usize] = [0.20, 0.22, 0.23, 1.00];
    styles.colors[StyleColor::TitleBg as usize] = [0.00, 0.00, 0.00, 1.00];
    styles.colors[StyleColor::TitleBgActive as usize] = [0.06, 0.06, 0.06, 1.00];
    styles.colors[StyleColor::TitleBgCollapsed as usize] = [0.00, 0.00, 0.00, 1.00];
    styles.colors[StyleColor::MenuBarBg as usize] = [0.14, 0.14, 0.14, 1.00];
    styles.colors[StyleColor::ScrollbarBg as usize] = [0.05, 0.05, 0.05, 0.54];
    styles.colors[StyleColor::ScrollbarGrab as usize] = [0.34, 0.34, 0.34, 0.54];
    styles.colors[StyleColor::ScrollbarGrabHovered as usize] = [0.40, 0.40, 0.40, 0.54];
    styles.colors[StyleColor::ScrollbarGrabActive as usize] = [0.56, 0.56, 0.56, 0.54];
    styles.colors[StyleColor::CheckMark as usize] = [0.86, 0.554, 0.33, 1.00];
    styles.colors[StyleColor::SliderGrab as usize] = [0.34, 0.34, 0.34, 0.54];
    styles.colors[StyleColor::SliderGrabActive as usize] = [0.56, 0.56, 0.56, 0.54];
    styles.colors[StyleColor::Button as usize] = [0.05, 0.05, 0.05, 0.54];
    styles.colors[StyleColor::ButtonHovered as usize] = [0.19, 0.19, 0.19, 0.54];
    styles.colors[StyleColor::ButtonActive as usize] = [0.20, 0.22, 0.23, 1.00];
    styles.colors[StyleColor::Header as usize] = [0.00, 0.00, 0.00, 0.52];
    styles.colors[StyleColor::HeaderHovered as usize] = [0.00, 0.00, 0.00, 0.36];
    styles.colors[StyleColor::HeaderActive as usize] = [0.20, 0.22, 0.23, 0.33];
    styles.colors[StyleColor::Separator as usize] = [0.28, 0.28, 0.28, 0.29];
    styles.colors[StyleColor::SeparatorHovered as usize] = [0.44, 0.44, 0.44, 0.29];
    styles.colors[StyleColor::SeparatorActive as usize] = [0.40, 0.44, 0.47, 1.00];
    styles.colors[StyleColor::ResizeGrip as usize] = [0.28, 0.28, 0.28, 0.29];
    styles.colors[StyleColor::ResizeGripHovered as usize] = [0.44, 0.44, 0.44, 0.29];
    styles.colors[StyleColor::ResizeGripActive as usize] = [0.40, 0.44, 0.47, 1.00];
    styles.colors[StyleColor::Tab as usize] = [0.00, 0.00, 0.00, 0.52];
    styles.colors[StyleColor::TabHovered as usize] = [0.14, 0.14, 0.14, 1.00];
    styles.colors[StyleColor::TabActive as usize] = [0.20, 0.20, 0.20, 0.36];
    styles.colors[StyleColor::TabUnfocused as usize] = [0.00, 0.00, 0.00, 0.52];
    styles.colors[StyleColor::TabUnfocusedActive as usize] = [0.14, 0.14, 0.14, 1.00];
    styles.colors[StyleColor::DockingPreview as usize] = [0.86, 0.554, 0.33, 1.00];
    styles.colors[StyleColor::NavHighlight as usize] = [0.86, 0.554, 0.33, 1.00];
    styles.colors[StyleColor::NavWindowingDimBg as usize] = [0.86, 0.554, 0.33, 1.00];
    styles.colors[StyleColor::NavWindowingHighlight as usize] = [0.86, 0.554, 0.33, 1.00];
    // styles.colors[StyleColor::DockingEmptyBg as usize]        = [1.00, 0.00, 0.00, 1.00];
    styles.colors[StyleColor::PlotLines as usize] = [1.00, 0.00, 0.00, 1.00];
    styles.colors[StyleColor::PlotLinesHovered as usize] = [1.00, 0.00, 0.00, 1.00];
    styles.colors[StyleColor::PlotHistogram as usize] = [1.00, 0.00, 0.00, 1.00];
    styles.colors[StyleColor::PlotHistogramHovered as usize] = [1.00, 0.00, 0.00, 1.00];
    styles.colors[StyleColor::TableHeaderBg as usize] = [0.00, 0.00, 0.00, 0.52];
    styles.colors[StyleColor::TableBorderStrong as usize] = [0.00, 0.00, 0.00, 0.52];
    styles.colors[StyleColor::TableBorderLight as usize] = [0.28, 0.28, 0.28, 0.29];
    styles.colors[StyleColor::TableRowBg as usize] = [0.00, 0.00, 0.00, 0.00];
    styles.colors[StyleColor::TableRowBgAlt as usize] = [1.00, 1.00, 1.00, 0.06];
    styles.colors[StyleColor::TextSelectedBg as usize] = [0.20, 0.22, 0.23, 1.00];
    styles.colors[StyleColor::DragDropTarget as usize] = [0.33, 0.67, 0.86, 1.00];
    // styles.colors[StyleColor::NavHighlight as usize] = [1.00, 0.00, 0.00, 1.00];
    // styles.colors[StyleColor::NavWindowingHighlight as usize] = [1.00, 0.00, 0.00, 0.70];
    // styles.colors[StyleColor::NavWindowingDimBg as usize] = [1.00, 0.00, 0.00, 0.20];
    styles.colors[StyleColor::ModalWindowDimBg as usize] = [1.00, 0.00, 0.00, 0.35];
}

pub mod ui {

    use imgui::Ui;

    pub fn text_label(ui: &Ui, label: &str) {
        let width = ui.calc_item_width();
        let pos = ui.cursor_pos();
        ui.text(label);
        ui.same_line();
        ui.set_cursor_pos([
            pos[0] + width * 0.5 + ui.clone_style().item_spacing[0],
            pos[1],
        ]);
        ui.set_next_item_width(-1.0);
    }

    pub fn input_text(ui: &Ui, label: &str, text: &mut String, hint: Option<&str>) -> bool {
        text_label(ui, label);
        let id = "##".to_string() + &label.to_string();
        let mut t = ui.input_text(id, text);
        if let Some(hint) = hint {
            t = t.hint(hint);
        }

        t.build()
    }
}
