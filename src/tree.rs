use std::{cell::RefCell, fmt::Debug, rc::Rc};

#[derive(PartialEq)]
struct TreeNodeInt<T> {
    object: T,
    childs: Vec<TreeNode<T>>,
    parent: Option<TreeNode<T>>,
}

#[derive(PartialEq)]
pub struct TreeNode<T> {
    node: Rc<RefCell<TreeNodeInt<T>>>,
}

impl<T> Clone for TreeNode<T> {
    fn clone(&self) -> Self {
        Self {
            node: Rc::clone(&self.node),
        }
    }
}

impl<T: Debug> Debug for TreeNode<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TreeNode")
            .field("node", &self.print())
            .finish()
    }
}

impl<T> TreeNode<T>
where
    T: Debug,
{
    pub fn new(value: T) -> TreeNode<T> {
        TreeNode {
            node: TreeNodeInt::new(value).pack(),
        }
    }

    pub fn add_child(&mut self, new_node: TreeNode<T>) {
        new_node.node.borrow_mut().parent = Some(self.clone());
        self.node.borrow_mut().add_child(new_node);
    }

    pub fn print(&self) -> String {
        self.node.borrow().print()
    }
}

impl<T> TreeNodeInt<T>
where
    T: Debug,
{
    fn new(value: T) -> TreeNodeInt<T> {
        TreeNodeInt {
            object: value,
            childs: vec![],
            parent: None,
        }
    }

    fn add_child(&mut self, new_node: TreeNode<T>) {
        self.childs.push(new_node);
    }

    fn print(&self) -> String {
        format!("{:?}", &self.object)
            + "["
            + &self
                .childs
                .iter()
                .map(|c| c.print())
                .collect::<Vec<String>>()
                .join(",")
            + "]"
    }

    fn pack(self) -> Rc<RefCell<TreeNodeInt<T>>> {
        Rc::new(RefCell::new(self))
    }
}
