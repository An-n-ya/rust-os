use core::{arch, cell::Cell};

use alloc::{collections::VecDeque, sync::Arc};

use crate::{
    hlt,
    interrupts::enable,
    sync::{lazy::Lazy, spin::SpinMutex},
};

use super::task::{x86_context_switch, X86Task};

pub static SCHEDULAR: Lazy<Arc<Scheduler>> = Lazy::new(|| Arc::new(Scheduler::new()));

pub struct Scheduler {
    queue: Arc<SpinMutex<VecDeque<Arc<X86Task>>>>,
    preempt_task: Arc<X86Task>,
    current_task: Arc<Cell<Arc<X86Task>>>,
}
unsafe impl Sync for Scheduler {}
unsafe impl Send for Scheduler {}

impl Scheduler {
    pub fn new() -> Self {
        let init_task = Arc::new(X86Task::new_kernel(kernel_1 as *const () as u64, 0));
        let task2 = Arc::new(X86Task::new_kernel(kernel_2 as *const () as u64, 3));
        let idle_task = Arc::new(X86Task::new_kernel(idle as *const () as u64, 1));
        let preempt_task = Arc::new(X86Task::new_kernel(preempt as *const () as u64, 2));
        let mut deque = VecDeque::new();
        deque.push_back(init_task);
        deque.push_back(task2);
        let queue = Arc::new(SpinMutex::new(deque));
        Self {
            queue,
            preempt_task,
            current_task: Arc::new(Cell::new(idle_task)),
        }
    }

    pub fn preempt(&self) {
        let current = unsafe { &*self.current_task.as_ref().as_ptr() };
        x86_context_switch(current.get_mut(), self.preempt_task.as_ref())
    }
}

fn preempt() {
    loop {
        let sched = &*SCHEDULAR;
        let mut queue = sched.queue.lock();
        let t = queue.pop_front();
        if let Some(task) = t {
            let prev = sched.current_task.replace(task.clone());
            queue.push_back(prev);

            drop(queue);

            x86_context_switch(sched.preempt_task.as_ref().get_mut(), task.as_ref())
        } else {
            panic!("cannot find task in scheduler queue")
        }
    }
}

fn kernel_1() {
    loop {
        for _ in 0..1000000 {
            unsafe { arch::asm!("nop") };
        }
        log!("kernel1");
    }
}
fn kernel_2() {
    loop {
        for _ in 0..1000000 {
            unsafe { arch::asm!("nop") };
        }
        log!("2222kernel2222");
    }
}

fn idle() {
    loop {
        enable();
        hlt();
    }
}
