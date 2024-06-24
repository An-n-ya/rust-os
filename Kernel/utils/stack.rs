use core::mem::size_of;

pub struct Stack<'a> {
    ptr: &'a mut usize,
}

impl<'a> Stack<'a> {
    pub fn new(ptr: &'a mut usize) -> Self {
        Self { ptr }
    }

    fn skip_by(&mut self, by: usize) {
        *self.ptr -= by;
    }

    pub unsafe fn offset<'b, T: Sized>(&mut self) -> &'b mut T {
        self.skip_by(size_of::<T>());
        &mut *(*self.ptr as *mut T)
    }

    pub unsafe fn push<T: Sized>(&mut self, value: T) {
        self.skip_by(size_of::<T>());
        (*self.ptr as *mut T).write(value)
    }
}
