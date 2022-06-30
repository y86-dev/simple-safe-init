use core::{mem::MaybeUninit, num::NonZeroU32};
use simple_safe_init::*;

pin_data! {
    #[derive(Debug)]
    pub struct Foo {
        a: u8,
        b: u64,
        c: NonZeroU32,
    }
}

fn main() {
    let foo = MaybeUninit::uninit();
    let foo = init! { foo => Foo {
        .a = 32;
        .b = *a as u64 + 7;
        .c = unsafe { NonZeroU32::new_unchecked(*b as u32) };
    }};
    println!("{:?}", foo);
}
