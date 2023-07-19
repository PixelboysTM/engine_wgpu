use std::{cell::RefCell, collections::HashMap, fmt::Debug, rc::Rc};

use super::{
    renderer::model::{Material, Mesh},
    Texture,
};

use crate::asset_type;

pub struct AssetDatabase {
    data: Rc<RefCell<InterDatabase>>,
}

asset_type!(Mesh, mesh, meshes, load_mesh);
asset_type!(Texture, texture, textures, load_texture);
asset_type!(Material, material, materials, load_material);

impl AssetDatabase {
    pub fn new() -> Self {
        AssetDatabase {
            data: Rc::new(RefCell::new(InterDatabase {
                textures: HashMap::new(),
                meshes: HashMap::new(),
                materials: HashMap::new(),
            })),
        }
    }
}

impl Clone for AssetDatabase {
    fn clone(&self) -> Self {
        Self {
            data: Rc::clone(&self.data),
        }
    }
}

struct InterDatabase {
    textures: HashMap<AssetLocation, AssetHandle<Texture>>,
    meshes: HashMap<AssetLocation, AssetHandle<Mesh>>,
    materials: HashMap<AssetLocation, AssetHandle<Material>>,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum AssetLocation {
    Builtin {
        idnetifying_name: &'static str,
    },
    Resource {
        path: String,
        in_file_ident: Option<String>,
    },
}

impl Debug for AssetLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.to_ident())
    }
}

impl AssetLocation {
    pub fn to_ident(&self) -> String {
        match self {
            AssetLocation::Builtin { idnetifying_name } => format!("builtin:{idnetifying_name}"),
            AssetLocation::Resource {
                path,
                in_file_ident,
            } => format!(
                "res:{path}{}",
                in_file_ident
                    .as_ref()
                    .map_or("".to_string(), |i| format!("#{i}"))
            ),
        }
    }
}

pub(crate) mod uuid {

    pub(crate) type Uuid = String;
    pub(crate) fn new_uuid() -> Uuid {
        let id = uuid::Uuid::new_v4();
        let time = std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .expect("Why system time before unix EPOCH")
            .as_millis();
        format!("{}-{:015}", id, time)
    }

    #[allow(dead_code)]
    pub(crate) fn empty() -> Uuid {
        format!("{}-{:015}", uuid::Uuid::nil(), 0u128)
    }
}

pub struct AssetHandle<T> {
    pub(crate) location: AssetLocation,
    pub(crate) asset: Rc<RefCell<T>>,
}
impl<T> Clone for AssetHandle<T> {
    fn clone(&self) -> Self {
        Self {
            location: self.location.clone(),
            asset: Rc::clone(&self.asset),
        }
    }
}

#[macro_use]
mod asset_helpers {

    #[macro_export]
    macro_rules! asset_type {
        ($type:ty, $name:ident, $var:ident, $load_name:ident) => {
            impl AssetDatabase {
                pub fn $name(&self, location: AssetLocation) -> Option<AssetHandle<$type>> {
                    self.data.borrow().$var.get(&location).map(|v| v.clone())
                }
                pub fn $load_name(
                    &self,
                    location: AssetLocation,
                    object: $type,
                ) -> AssetHandle<$type> {
                    let handle = AssetHandle {
                        location: location.clone(),
                        asset: Rc::new(RefCell::new(object)),
                    };
                    self.data.borrow_mut().$var.insert(location, handle.clone());
                    handle
                }
            }
        };
    }
}
