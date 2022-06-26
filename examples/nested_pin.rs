use core::mem::MaybeUninit;
use easy_init::*;

pin_data! {
    #[derive(Debug)]
    pub struct Foo {
        a: u8,
        #pin
        b: Bar,
    }
}

#[derive(Debug)]
pub struct Bar {
    val: u8,
    my_addr: usize,
}

fn init_bar<G>(bar: PinInitMe<'_, Bar, G>) -> InitProof<(), G> {
    init! { bar => Bar {
        .val = 42;
        .my_addr = 0;
    }}
}

fn main() {
    let foo = Box::pin(MaybeUninit::uninit());
    let foo = init! { foo => Foo {
        .a = 32;
        init_bar(.b);
    }};
    println!("{:?}", foo);
}
