#![feature(allocator_api)]
use core::pin::Pin;
use simple_safe_init::*;
mod structs {
    use core::{marker::PhantomPinned, pin::Pin};
    use simple_safe_init::*;
    #[derive(Debug)]
    pub struct MyPinnedStruct {
        msg: String,
        // this will be our field that depends upon the pinning
        my_addr: usize,
        _p: PhantomPinned,
    }
    impl MyPinnedStruct {
        // a method that only works, if we are pinned
        pub fn print_info(self: Pin<&mut Self>) {
            println!("'{}' says MyPinnedStruct at {:X}", self.msg, self.my_addr);
        }
        // this is an init function, it takes a `PinInitMe` as its first
        // argument and returns an `InitProof` verifying the initialization.
        // The generic argument `G` is called the guard type, it is needed to ensure soundness
        // of the library.
        //
        // you can have any number of additional arguments
        pub fn init<G: Guard>(mut this: PinInitMe<'_, Self, G>, msg: String) -> InitProof<(), G> {
            // we still need the address for this example
            let addr = this.as_mut_ptr() as usize;
            // we still use the same syntax here!
            init! { this => Self {
                ._p = PhantomPinned;
                .msg = msg;
                .my_addr = addr;
            }}
        }
    }
}
use structs::MyPinnedStruct;
fn main() -> Result<(), std::alloc::AllocError> {
    let mut my_struct =
        init!(@MyPinnedStruct::init(Pin<Box<MyPinnedStruct>>, "Hello World".to_owned()))?;
    my_struct.as_mut().print_info();
    Ok::<(), std::alloc::AllocError>(())
}
