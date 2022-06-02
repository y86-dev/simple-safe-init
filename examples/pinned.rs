use core::mem::MaybeUninit;
use easy_init::*;

#[derive(Debug)]
pub struct Foo {
    a: u8,
    b: usize,
}

fn main() {
    let foo = Box::pin(MaybeUninit::uninit());
    let b = &*foo as *const MaybeUninit<Foo> as usize;
    let foo = init! { foo => Foo {
        .a = 32;
        .b = b;
    }};
    println!("{:?}", foo);
}
