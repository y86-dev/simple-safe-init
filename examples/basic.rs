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

fn generate() -> u64 {
    64 + 8
}

fn main() {
    let foo = MaybeUninit::uninit();
    let foo = init! { foo => Foo {
        .a = 32;
        .b = generate();
        .c = unsafe { NonZeroU32::new_unchecked(9) };
    }};
    println!("{:?}", foo);
}
