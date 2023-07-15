use std::{cell::RefCell, rc::Rc};

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
    pub(crate) fn empty() -> Uuid {
        format!("{}-{:015}", uuid::Uuid::nil(), 0u128)
    }
}

pub struct AssetHandle<T> {
    pub(crate) asset: Rc<RefCell<T>>,
}
impl<T> Clone for AssetHandle<T> {
    fn clone(&self) -> Self {
        Self {
            asset: Rc::clone(&self.asset),
        }
    }
}
