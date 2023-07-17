mod mesh_filter;
mod transform;

use std::{cell::RefCell, collections::HashMap, rc::Rc};

use imgui::Ui;
pub use mesh_filter::*;
use serde::{Deserialize, Serialize};
pub use transform::*;

use super::SceneObject;

#[derive(Serialize, Deserialize, PartialEq)]
pub enum Component {
    MeshFilter(MeshFilter),
}

pub trait ComponentPacker {
    fn pack(self) -> Component;
}

pub type ComponentIdentifier = &'static str;

#[derive(PartialEq)]
pub struct ComponentHandle {
    inter: Rc<RefCell<Component>>, //TODO: Make to weak
}

impl Serialize for ComponentHandle {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.inter.borrow().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ComponentHandle {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let c = Component::deserialize(deserializer)?;
        Ok(ComponentHandle {
            inter: Rc::new(RefCell::new(c)),
        })
    }
}

impl Clone for ComponentHandle {
    fn clone(&self) -> Self {
        Self {
            inter: Rc::clone(&self.inter),
        }
    }
}

impl ComponentHandle {
    pub fn get(&self) -> &Rc<RefCell<Component>> {
        &self.inter
    }
}

#[derive(Serialize, Deserialize, PartialEq)]
pub(super) struct ComponentContainer {
    pub(super) transform: Transform,
    components: HashMap<String, ComponentHandle>,

    #[serde(skip)]
    object: Option<SceneObject>,
}

impl ComponentContainer {
    pub(super) fn empty() -> ComponentContainer {
        ComponentContainer {
            components: HashMap::new(),
            transform: Transform::default(),
            object: None,
        }
    }

    pub(super) fn attach(&mut self, object: SceneObject) {
        self.object = Some(object);
    }

    pub(super) fn gui(&mut self, ui: &Ui) {
        self.transform.gui(ui);
        for (_, component) in self.components.iter_mut() {
            component.inter.borrow_mut().gui(ui);
        }
    }

    pub fn add_component<T>(&mut self, component: T)
    where
        T: ComponentPacker,
    {
        let mut comp = component.pack();
        let ident = comp.ident().to_string();
        comp.attach(self.object.as_ref().expect("Must be attached").clone());
        let comp = comp.pack();

        self.components.insert(ident, comp);
    }
    pub fn get_component(&self, identifier: ComponentIdentifier) -> Option<ComponentHandle> {
        if self.components.contains_key(identifier) {
            let c = self.components.get(identifier).expect("testet with if");
            Some(c.clone())
        } else {
            None
        }
    }
}

impl Component {
    fn gui(&mut self, ui: &Ui) {
        match self {
            Component::MeshFilter(mesh_filter) => mesh_filter.gui(ui),
        }
    }
    fn ident(&self) -> &'static str {
        match self {
            Component::MeshFilter(_) => MeshFilter::IDENT,
        }
    }
    fn pack(self) -> ComponentHandle {
        ComponentHandle {
            inter: Rc::new(RefCell::new(self)),
        }
    }

    fn attach(&mut self, object: SceneObject) {
        match self {
            Component::MeshFilter(filter) => filter.attach(object),
        }
    }
}

impl ComponentPacker for Component {
    fn pack(self) -> Component {
        self
    }
}
