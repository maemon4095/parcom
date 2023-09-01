use std::{rc::Rc, sync::Arc};

pub trait Shared<T>: AsRef<T> + Clone {
    fn get_mut(&mut self) -> Option<&mut T>;
}

pub trait WeakRef<T>: Clone {
    type Upgraded: Shared<T>;
    fn upgrade(&self) -> Option<Self::Upgraded>;
}

impl<T> Shared<T> for Rc<T> {
    fn get_mut(&mut self) -> Option<&mut T> {
        Rc::get_mut(self)
    }
}

impl<T> WeakRef<T> for std::rc::Weak<T> {
    type Upgraded = Rc<T>;

    fn upgrade(&self) -> Option<Self::Upgraded> {
        std::rc::Weak::upgrade(self)
    }
}

impl<T> Shared<T> for Arc<T> {
    fn get_mut(&mut self) -> Option<&mut T> {
        Arc::get_mut(self)
    }
}

impl<T> WeakRef<T> for std::sync::Weak<T> {
    type Upgraded = Arc<T>;

    fn upgrade(&self) -> Option<Self::Upgraded> {
        std::sync::Weak::upgrade(self)
    }
}
