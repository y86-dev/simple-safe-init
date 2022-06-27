#![feature(generic_associated_types)]
use easy_init::*;
use core::mem::MaybeUninit;

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
