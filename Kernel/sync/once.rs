use core::{
    cell::UnsafeCell,
    mem::MaybeUninit,
    sync::atomic::{AtomicU8, Ordering},
};

pub struct Once<T> {
    status: AtomicStatus,
    data: UnsafeCell<MaybeUninit<T>>,
}

#[repr(transparent)]
struct AtomicStatus(AtomicU8);

#[repr(u8)]
#[derive(Clone, Copy, PartialEq)]
enum Status {
    Incomplete = 0x00,
    Running = 0x01,
    Complete = 0x02,
    Panicked = 0x03,
}

impl<T> Default for Once<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Once<T> {
    pub const fn new() -> Self {
        Self {
            status: AtomicStatus::new(Status::Incomplete),
            data: UnsafeCell::new(MaybeUninit::uninit()),
        }
    }
    pub fn get(&self) -> Option<&T> {
        match self.status.load(Ordering::Acquire) {
            Status::Complete => Some(unsafe { &*(*self.data.get()).as_ptr() }),
            _ => None,
        }
    }
    pub fn call_once<F: FnOnce() -> Result<T, E>, E>(&self, f: F) -> Result<&T, E> {
        if let Some(value) = self.get() {
            Ok(value)
        } else {
            loop {
                let prev_status = self.status.compare_exchange(
                    Status::Incomplete,
                    Status::Running,
                    Ordering::Acquire,
                    Ordering::Acquire,
                );
                match prev_status {
                    Ok(_) => {
                        // expect
                    }
                    Err(Status::Panicked) => panic!("Once panicked"),
                    Err(Status::Running) => match self.poll() {
                        None => continue,
                        Some(val) => return Ok(val),
                    },
                    Err(Status::Complete) => return Ok(self.get().unwrap()),
                    Err(Status::Incomplete) => {
                        unreachable!("reach incomplete in Once")
                    }
                }
                let val = match f() {
                    Ok(val) => val,
                    Err(err) => {
                        self.status.store(Status::Incomplete, Ordering::Release);
                        return Err(err);
                    }
                };
                unsafe {
                    (*self.data.get()).as_mut_ptr().write(val);
                }
                self.status.store(Status::Complete, Ordering::Release);
                return Ok(self.get().unwrap());
            }
        }
    }
    pub fn poll(&self) -> Option<&T> {
        loop {
            match self.status.load(Ordering::Acquire) {
                Status::Incomplete => return None,
                Status::Running => {} // wait
                Status::Complete => return self.get(),
                Status::Panicked => panic!("Poll meets Panic in call_once"),
            }
        }
    }
}
impl<T> Drop for Once<T> {
    fn drop(&mut self) {
        if self.status.load(Ordering::Relaxed) == Status::Complete {
            unsafe {
                core::ptr::drop_in_place((*self.data.get()).as_mut_ptr());
            }
        }
    }
}

impl Status {
    unsafe fn new_unchecked(inner: u8) -> Self {
        core::mem::transmute(inner)
    }
}

impl AtomicStatus {
    pub const fn new(status: Status) -> Self {
        Self(AtomicU8::new(status as u8))
    }

    pub fn load(&self, ordering: Ordering) -> Status {
        unsafe { Status::new_unchecked(self.0.load(ordering)) }
    }
    pub fn store(&self, status: Status, ordering: Ordering) {
        self.0.store(status as u8, ordering);
    }
    pub fn compare_exchange(
        &self,
        old: Status,
        new: Status,
        success: Ordering,
        failure: Ordering,
    ) -> Result<Status, Status> {
        match self
            .0
            .compare_exchange(old as u8, new as u8, success, failure)
        {
            Ok(ok) => Ok(unsafe { Status::new_unchecked(ok) }),
            Err(err) => Err(unsafe { Status::new_unchecked(err) }),
        }
    }
}
