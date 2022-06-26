use core::mem::MaybeUninit;
use easy_init::*;

pin_data! {
    #[derive(Debug)]
    pub struct Foo {
        a: u8,
        b: usize,
    }
}

fn init_usize<G>(val: InitMe<'_, usize, G>) -> InitProof<(), G> {
    val.write(5)
}

fn main() {
    let foo = Box::pin(MaybeUninit::uninit());
    let foo = init! { foo => Foo {
        .a = 32;
        init_usize(.b);
    }};
    println!("{:?}", foo);
}
