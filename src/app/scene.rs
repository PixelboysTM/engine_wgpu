pub mod component;

use std::{cell::RefCell, rc::Rc};

use imgui::Ui;
use serde::{Deserialize, Serialize};

use crate::gui::ui;

use self::component::{
    ComponentContainer, ComponentHandle, ComponentIdentifier, ComponentPacker, Transform,
};

use super::assets::uuid::{Uuid, *};

pub struct Scene {
    inter: Rc<RefCell<InterScene>>,
}

#[derive(Serialize, Deserialize)]
struct InterScene {
    name: String,
    uuid: Uuid,
    root: SceneObject,

    //Payload for internal function
    #[serde(skip)]
    selected: Option<Uuid>,
}

impl Serialize for Scene {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.inter.borrow().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Scene {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let inter = InterScene::deserialize(deserializer)?;
        Ok(Scene {
            inter: Rc::new(RefCell::new(inter)),
        }
        .reparent())
    }
}

impl Clone for Scene {
    fn clone(&self) -> Self {
        Self {
            inter: Rc::clone(&self.inter),
        }
    }
}

impl Scene {
    pub fn new<S: Into<String>>(name: S) -> Scene {
        let id = new_uuid();
        let name = name.into();
        let root = SceneObject::new(name.clone());
        Scene {
            inter: Rc::new(RefCell::new(InterScene {
                root,
                name,
                uuid: id,
                // payload
                selected: None,
            })),
        }
    }
    pub fn root(&self) -> SceneObject {
        let inter = self.inter.borrow();
        inter.root.clone()
    }

    pub fn name(&self) -> String {
        self.inter.borrow().name.clone()
    }
    pub fn find(&self, id: &Uuid) -> Option<SceneObject> {
        fn find(obj: &SceneObject, id: &Uuid) -> Option<SceneObject> {
            if &obj.id() == id {
                return Some(obj.clone());
            } else {
                obj.children().iter().find_map(|f| find(f, id))
            }
        }
        find(&self.root(), id)
    }

    fn reparent(self) -> Self {
        fn reparent_obj(obj: &SceneObject) {
            obj.children().iter().for_each(|c| {
                c.inter.borrow_mut().parent = Some(obj.clone());
                reparent_obj(c);
            })
        }

        reparent_obj(&self.inter.borrow().root);
        self
    }
}

impl Scene {
    pub fn gui(&self, ui: &mut Ui) {
        {
            let hierachy = ui.window("Hierachy").begin();
            if hierachy.is_some() {
                ui.text(&format!("Name: {}", self.name()));
                ui.separator();
                Self::scene_object_hierachy(
                    ui,
                    &self.root(),
                    &mut self.inter.borrow_mut().selected,
                );
            }
        }
        {
            let inspector = ui.window("Inspector").begin();
            if inspector.is_some() {
                if let Some(selected) = &self.inter.borrow().selected {
                    let scene_object = self.find(selected).expect("Was set before");
                    scene_object.draw_inspector(ui);
                }
            }
        }
    }
    fn scene_object_hierachy(ui: &Ui, scene_object: &SceneObject, selected: &mut Option<Uuid>) {
        let is_leaf = scene_object.child_count() == 0;
        let node_open = ui
            .tree_node_config::<String, String>(scene_object.id())
            .default_open(true)
            .label::<String, String>(scene_object.name())
            .selected(selected.clone().map_or(false, |id| id == scene_object.id()))
            .open_on_double_click(true)
            .leaf(is_leaf)
            .push();
        if let Some(_) = node_open {
            if ui.is_item_clicked() {
                *selected = Some(scene_object.id());
            }
            if ui.is_item_clicked_with_button(imgui::MouseButton::Right) {
                ui.open_popup("hierachy_scene_context_popup");
            }
            {
                let popup = ui.begin_popup("hierachy_scene_context_popup");
                if let Some(_) = popup {
                    ui.menu_item("Test");
                    ui.checkbox("Opened", &mut false);
                }
            }

            for child in scene_object.children() {
                Scene::scene_object_hierachy(ui, &child, selected);
            }
        }
    }
}

#[derive(PartialEq, Serialize, Deserialize)]
struct InterSceneObject {
    name: String,
    uuid: Uuid,
    components: ComponentContainer,

    childs: Vec<SceneObject>,
    #[serde(skip)]
    parent: Option<SceneObject>,
}

// impl Debug for InterSceneObject {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         f.debug_struct("InterSceneObject")
//             .field("childs", &self.childs)
//             .field("parent", &self.parent)
//             .finish()
//     }
// }

#[derive(PartialEq)]
pub struct SceneObject {
    inter: Rc<RefCell<InterSceneObject>>,
}

impl Serialize for SceneObject {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.inter.borrow().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for SceneObject {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let inter = InterSceneObject::deserialize(deserializer)?;
        Ok(SceneObject {
            inter: Rc::new(RefCell::new(inter)),
        })
    }
}

impl Clone for SceneObject {
    fn clone(&self) -> Self {
        Self {
            inter: Rc::clone(&self.inter),
        }
    }
}

impl SceneObject {
    pub fn new<S: Into<String>>(name: S) -> SceneObject {
        let s = SceneObject {
            inter: Rc::new(RefCell::new(InterSceneObject {
                name: name.into(),
                uuid: new_uuid(),
                childs: vec![],
                parent: None,
                components: ComponentContainer::empty(),
            })),
        };
        s.inter.borrow_mut().components.attach(s.clone());
        s
    }

    pub fn parent(&self) -> Option<SceneObject> {
        if self.has_parent() {
            self.inter.borrow().parent.clone()
        } else {
            None
        }
    }
    pub fn has_parent(&self) -> bool {
        self.inter.borrow().parent.is_some()
    }
    pub fn id(&self) -> Uuid {
        self.inter.borrow().uuid.clone()
    }
    pub fn name(&self) -> String {
        self.inter.borrow().name.clone()
    }
    pub fn remove_child(&self, child: &SceneObject) {
        let childs = &mut self.inter.borrow_mut().childs;
        if childs.contains(child) {
            let index = childs
                .iter()
                .position(|c| child == c)
                .expect("Tested if in vec");
            childs.remove(index);
        }
    }
    pub fn children(&self) -> Vec<SceneObject> {
        self.inter.borrow().childs.clone()
    }
    pub fn child_count(&self) -> usize {
        self.inter.borrow().childs.len()
    }

    pub fn add_child(&self, child: SceneObject) {
        if let Some(parent) = child.parent() {
            parent.remove_child(self);
        }
        child.inter.borrow_mut().parent = Some(self.clone());
        self.inter.borrow_mut().childs.push(child);
    }

    pub fn get_transform(&self) -> Transform {
        self.inter.borrow().components.transform.clone()
    }
}

impl SceneObject {
    pub fn add_component<T>(&self, component: T)
    where
        T: ComponentPacker,
    {
        self.inter.borrow_mut().components.add_component(component);
    }

    pub fn get_component(&self, identifier: ComponentIdentifier) -> Option<ComponentHandle> {
        let inter = &self.inter.borrow().components;
        inter.get_component(identifier)
    }
}

impl SceneObject {
    fn draw_inspector(&self, ui: &Ui) {
        let mut inter = self.inter.borrow_mut();
        // ui.input_text("Name:", &mut inter.name).hint("Name").build();
        ui::text_label(ui, "Id:");
        ui.text_disabled(&inter.uuid);
        ui::input_text(ui, "Name:", &mut inter.name, Some("Name"));
        ui.separator();
        inter.components.gui(ui);
    }
}
