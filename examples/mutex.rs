#![feature(generic_associated_types)]
use core::{cell::UnsafeCell, marker::PhantomPinned, mem::MaybeUninit};
use easy_init::*;

#[derive(Debug)]
#[repr(C)]
struct mutex {
    data: [u8; 16],
}

// extern
unsafe fn __init_mutex(_mutex: *mut mutex) {}

fn init_raw_mutex<G>(mut mutex: PinInitMe<'_, mutex, G>) -> InitProof<(), G> {
    unsafe {
        __init_mutex(mutex.as_mut_ptr());
        mutex.assume_init()
    }
}

pin_data! {
    #[derive(Debug)]
    pub struct Mutex<T> {
        #pin
        raw: mutex,
        #pin
        pin: PhantomPinned,
        val: UnsafeCell<T>,
    }
}

fn create_single_mutex() {
    let mtx = Box::pin(MaybeUninit::uninit());
    let mtx = init! { mtx => Mutex<String> {
        init_raw_mutex(.raw);
        .pin = PhantomPinned;
        .val = UnsafeCell::new("Hello World".to_owned());
    }};
    println!("{:?}", mtx);
}

pin_data! {
    #[derive(Debug)]
    struct MultiMutex {
        #pin
        data1: Mutex<String>,
        #pin
        data2: Mutex<(u64, f64)>,
    }
}

fn init_mutex<T, G>(mutex: PinInitMe<'_, Mutex<T>, G>, value: T) -> InitProof<(), G> {
    init! { mutex => Mutex<T> {
        init_raw_mutex(.raw);
        .val = UnsafeCell::new(value);
        .pin = PhantomPinned;
    }}
}

fn create_multi_mutex() {
    let mmx = Box::pin(MaybeUninit::<MultiMutex>::uninit());
    let mmx = init! { mmx => MultiMutex {
        init_mutex(.data1, "Hello World".to_owned());
        init_mutex(.data2, (42, 13.37));
    }};
    println!("{:?}", mmx);
}

fn main() {
    create_single_mutex();
    create_multi_mutex();
}
