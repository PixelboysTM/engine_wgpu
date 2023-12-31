pub(crate) mod assets;
mod renderer;
mod scene;

use std::time::Duration;

use winit::{
    dpi::PhysicalSize,
    event::{Event, WindowEvent},
    window::Window,
};

pub(crate) use renderer::texture::*;

use crate::app::scene::{component::MeshFilter, SceneObject};

use self::{
    assets::{AssetDatabase, AssetLocation},
    renderer::{model::load_model, Renderer},
    scene::Scene,
};

pub struct ApplicationState {
    renderer: Renderer,
    size: winit::dpi::PhysicalSize<u32>,
    window: Window,
    scene: Scene,
    asset_db: AssetDatabase,
    // obj_model: Model,
}

impl ApplicationState {
    pub(super) async fn new(window: Window) -> Self {
        let size = window.inner_size();

        let renderer = Renderer::new(&window, size).await;
        let asset_db = AssetDatabase::new();

        let obj_model = load_model(
            "cube.obj",
            renderer.device(),
            renderer.queue(),
            asset_db.clone(),
        )
        .await
        .unwrap();
        let model_loc = AssetLocation::Resource {
            path: "cube.json".to_owned(),
            in_file_ident: None,
        };
        let obj_model = asset_db.load_model(model_loc, obj_model);

        // const SPACE_BETWEEN: f32 = 3.0;
        // let instances = (0..NUM_INSTANCES_PER_ROW)
        //     .flat_map(|z| {
        //         (0..NUM_INSTANCES_PER_ROW).map(move |x| {
        //             let x = SPACE_BETWEEN * (x as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0);
        //             let z = SPACE_BETWEEN * (z as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0);

        //             let position = cgmath::Vector3 { x, y: 0.0, z };

        //             let rotation = if position.is_zero() {
        //                 cgmath::Quaternion::from_axis_angle(
        //                     cgmath::Vector3::unit_z(),
        //                     cgmath::Deg(0.0),
        //                 )
        //             } else {
        //                 cgmath::Quaternion::from_axis_angle(position.normalize(), cgmath::Deg(45.0))
        //             };

        //             Instance { position, rotation }
        //         })
        //     })
        //     .collect::<Vec<_>>();

        // let instance_data = instances.iter().map(Instance::to_raw).collect::<Vec<_>>();

        let scene = Scene::new("Test Scene");
        let root = scene.root();
        root.add_component(MeshFilter::with_material(
            obj_model.asset().meshes[0].clone(),
            obj_model.asset().materials[0].clone(),
        ));
        root.add_child(SceneObject::new("SceneObject 1"));
        root.add_child(SceneObject::new("SceneObject 2"));
        root.add_child(SceneObject::new("SceneObject 3"));
        let obj = SceneObject::new("SceneObject 4");
        obj.add_child(SceneObject::new("Child 1"));
        let obj2 = SceneObject::new("Child 2");
        obj2.add_child(SceneObject::new("Subchild 1"));
        obj2.add_child(SceneObject::new("Subchild 2"));
        obj2.add_child(SceneObject::new("Subchild 3"));

        obj2.add_component(MeshFilter::with_material(
            obj_model.asset().meshes[0].clone(),
            obj_model.asset().materials[0].clone(),
        ));
        obj.add_child(obj2);
        obj.add_child(SceneObject::new("Child 3"));
        obj.add_child(SceneObject::new("Child 4"));
        root.add_child(obj);

        // let yml = serde_yaml::to_string(&scene).unwrap();
        // println!("{}", yml);

        Self {
            renderer,
            size,
            window,
            // obj_model,
            // instances,
            // instance_buffer,
            scene,
            asset_db,
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub(super) fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.renderer.resize(new_size);
        }
    }

    #[allow(unused_variables)]
    pub(super) fn input(&mut self, event: &WindowEvent) -> bool {
        false
    }

    pub(super) fn event<T>(&mut self, event: &Event<T>) {
        self.renderer.event(&self.window, event);
    }

    pub(super) fn update(&mut self, dt: Duration) {
        self.renderer.update(dt, &self.window, &self.scene);
    }

    pub(super) fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        self.renderer.render(&self.scene)
    }

    pub(super) fn size(&self) -> &PhysicalSize<u32> {
        &self.size
    }
}
