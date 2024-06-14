use core::{
    cell::{Cell, OnceCell},
    ops::Deref,
};

pub struct Lazy<T, F = fn() -> T> {
    cell: OnceCell<T>,
    init: Cell<Option<F>>,
}
unsafe impl<T: Send> Sync for Lazy<T> {}
unsafe impl<T: Send> Send for Lazy<T> {}

impl<T, F> Lazy<T, F> {
    pub const fn new(init: F) -> Lazy<T, F> {
        Lazy {
            cell: OnceCell::new(),
            init: Cell::new(Some(init)),
        }
    }
}

impl<T, F: FnOnce() -> T> Lazy<T, F> {
    pub fn force(this: &Lazy<T, F>) -> &T {
        this.cell.get_or_init(|| match this.init.take() {
            Some(f) => f(),
            None => panic!("Lazy instance has previously been poisoned"),
        })
    }
}

impl<T, F: FnOnce() -> T> Deref for Lazy<T, F> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        Lazy::force(self)
    }
}

impl<T: Default> Default for Lazy<T> {
    fn default() -> Self {
        Lazy::new(T::default)
    }
}
