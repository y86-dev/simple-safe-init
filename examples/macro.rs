#![feature(generic_associated_types)]
use core::mem::MaybeUninit;
use simple_safe_init::*;

macro_rules! init_int {
    ($var:ident) => {
        $var.write(0)
    };
}

pin_data! {
    struct Foo {
        a: usize,
    }
}

fn main() {
    let foo = Box::pin(MaybeUninit::<Foo>::uninit());
    let foo = init! { foo => Foo {
        init_int!(.a);
    }};
}
