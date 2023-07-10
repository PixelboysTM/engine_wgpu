mod app;
mod gui;
mod tree;

use std::time::Instant;

use app::ApplicationState;
use tree::TreeNode;
use winit::{
    dpi::PhysicalSize,
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::EventLoop,
    platform::windows::WindowBuilderExtWindows,
    window::{Icon, WindowBuilder},
};

const ICON_DATA: &'static [u8] = include_bytes!("icon.png");
const ICON_DATA_BIG: &'static [u8] = include_bytes!("icon_big.png");

fn main() {
    env_logger::init();
    pollster::block_on(run());
}

async fn run() {
    #[cfg(feature = "render-doc")]
    let mut rd: renderdoc::RenderDoc<renderdoc::V141> =
        renderdoc::RenderDoc::new().expect("Unable to connect Renderdoc");

    #[cfg(feature = "render-doc")]
    rd.start_frame_capture(std::ptr::null(), std::ptr::null());

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(PhysicalSize::new(800, 600))
        .with_resizable(true)
        .with_theme(Some(winit::window::Theme::Dark))
        .with_title("Moinchen")
        .with_window_icon(Some(make_icon(ICON_DATA)))
        .with_taskbar_icon(Some(make_icon(ICON_DATA_BIG)))
        .build(&event_loop)
        .unwrap();

    let mut state = ApplicationState::new(window).await;

    tree();

    let mut last_frame = Instant::now();

    event_loop.run(move |event, _, contrlo_flow| {
        state.event(&event);
        match event {
            Event::WindowEvent {
                window_id,
                ref event,
            } if state.window().id() == window_id => {
                if !state.input(event) {
                    match event {
                        WindowEvent::CloseRequested
                        | WindowEvent::KeyboardInput {
                            input:
                                KeyboardInput {
                                    state: ElementState::Pressed,
                                    virtual_keycode: Some(VirtualKeyCode::Escape),
                                    ..
                                },
                            ..
                        } => {
                            contrlo_flow.set_exit();
                            #[cfg(feature = "render-doc")]
                            rd.end_frame_capture(std::ptr::null(), std::ptr::null());
                            // Todo: Into own object with drop
                        }
                        WindowEvent::Resized(physical_size) => {
                            state.resize(*physical_size);
                        }
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            state.resize(**new_inner_size);
                        }
                        _ => {}
                    }
                }
            }
            Event::RedrawRequested(window_id) if window_id == state.window().id() => {
                let delta_s = last_frame.elapsed();
                last_frame = Instant::now();
                state.update(delta_s);
                match state.render() {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::Lost) => state.resize(*state.size()),
                    Err(wgpu::SurfaceError::OutOfMemory) => contrlo_flow.set_exit(),
                    Err(e) => eprintln!("{:?}", e),
                }
            }
            Event::MainEventsCleared => {
                state.window().request_redraw();
            }
            _ => {}
        }
    })
}

fn make_icon(bytes: &[u8]) -> Icon {
    let img = image::load_from_memory(bytes).unwrap();
    Icon::from_rgba(img.to_rgba8().to_vec(), img.width(), img.height()).unwrap()
}

fn tree() {
    let mut root = TreeNode::new("Root");
    root.add_child(TreeNode::new("Child1"));
    root.add_child(TreeNode::new("Child2"));
    root.add_child(TreeNode::new("Child3"));
    let mut child4 = TreeNode::new("Child4");
    root.add_child(child4.clone());
    child4.add_child(TreeNode::new("Child4.1"));
    child4.add_child(TreeNode::new("Child4.2"));

    println!("Tree: {}", root.print());
}
