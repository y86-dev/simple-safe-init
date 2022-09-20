#![feature(
    type_alias_impl_trait,
    never_type,
    try_blocks,
    stmt_expr_attributes,
    raw_ref_op,
    new_uninit,
    unwrap_infallible
)]
use core::{
    cell::{Cell, UnsafeCell},
    ops::{Deref, DerefMut},
    sync::atomic::{AtomicBool, Ordering},
};
use std::{
    sync::Arc,
    thread::{self, Thread},
};

use simple_safe_init::*;
#[allow(unused_attributes)]
pub mod linked_list;
use linked_list::*;

pub struct SpinLock {
    inner: AtomicBool,
}

impl SpinLock {
    #[inline]
    pub fn acquire(&self) -> SpinLockGuard<'_> {
        while self
            .inner
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {}
        SpinLockGuard(self)
    }

    #[inline]
    pub fn new() -> Self {
        Self {
            inner: AtomicBool::new(false),
        }
    }
}

pub struct SpinLockGuard<'a>(&'a SpinLock);

impl Drop for SpinLockGuard<'_> {
    #[inline]
    fn drop(&mut self) {
        self.0.inner.store(false, Ordering::Release);
    }
}

pub struct Mutex<T> {
    wait_list: ListHead,
    spin_lock: SpinLock,
    locked: Cell<bool>,
    data: UnsafeCell<T>,
}

impl<T> Mutex<T> {
    #[inline]
    pub const fn new(val: T) -> impl Initializer<Self, !> {
        init!( <- Self {
            wait_list <- ListHead::new(),
            spin_lock: SpinLock::new(),
            locked: Cell::new(false),
            data: UnsafeCell::new(val),
        })
    }

    #[inline]
    pub fn lock(&self) -> MutexGuard<'_, T> {
        let mut sguard = self.spin_lock.acquire();
        if self.locked.get() {
            Result::<(), !>::into_ok(
                try {
                    stack_init!(let wait_entry <- WaitEntry::insert_new(&self.wait_list));
                    while self.locked.get() {
                        drop(sguard);
                        thread::park();
                        sguard = self.spin_lock.acquire();
                    }
                    drop(wait_entry);
                },
            );
        }
        self.locked.set(true);
        MutexGuard { mtx: self }
    }
}

unsafe impl<T: Send> Send for Mutex<T> {}
unsafe impl<T: Send> Sync for Mutex<T> {}

pub struct MutexGuard<'a, T> {
    mtx: &'a Mutex<T>,
}

impl<'a, T> Drop for MutexGuard<'a, T> {
    #[inline]
    fn drop(&mut self) {
        let sguard = self.mtx.spin_lock.acquire();
        self.mtx.locked.set(false);
        if let Some(list_field) = self.mtx.wait_list.next() {
            let wait_entry = list_field.as_ptr().cast::<WaitEntry>();
            unsafe { (*wait_entry).thread.unpark() };
        }
        drop(sguard);
    }
}

impl<'a, T> Deref for MutexGuard<'a, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.mtx.data.get() }
    }
}

impl<'a, T> DerefMut for MutexGuard<'a, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.mtx.data.get() }
    }
}

#[repr(C)]
struct WaitEntry {
    wait_list: ListHead,
    thread: Thread,
}

impl WaitEntry {
    #[inline]
    fn insert_new(list: &ListHead) -> impl Initializer<Self, !> + '_ {
        init!(<- Self {
            thread: thread::current(),
            wait_list <- ListHead::insert_new(list),
        })
    }
}

fn main() -> Result<(), AllocInitErr<!>> {
    let mtx = Arc::pin_init(Mutex::new(0))?;
    let mtx2 = mtx.clone();
    let t = thread::spawn(move || {
        *mtx2.lock() = 1;
        while *mtx2.lock() == 1 {}
        for _ in 0..1000_000 {
            *mtx2.lock() += 1;
        }
    });
    while *mtx.lock() != 1 {}
    *mtx.lock() = 0;
    for _ in 0..1000_000 {
        *mtx.lock() -= 1;
    }
    t.join().expect("thread panicked");
    println!("{}", &*mtx.lock());
    Ok(())
}
