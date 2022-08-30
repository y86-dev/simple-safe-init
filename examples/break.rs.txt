use core::mem::MaybeUninit;
use simple_safe_init::*;

pin_data! {
    #[derive(Debug)]
    struct Foo {
        #pin
        msg: String,
    }
}

pin_data! {
    #[derive(Debug)]
    struct Baz {
        #pin
        foo: Foo,
    }
}

pin_data! {
    #[derive(Debug)]
    struct Bar2 {
        #pin
        bar: Foo,
    }
}

fn evil<G>(foo: PinInitMe<'_, Foo, G>) -> InitProof<(), G> {
    let baz = Box::pin(MaybeUninit::<Baz>::uninit());
    let baz = init! { baz => Baz {
        init_bar(.foo, foo);
    }};
    println!("{:?}", baz);
    todo!()
}

fn init_bar<'a, 'b, G>(
    _foo: PinInitMe<'a, Foo, G>,
    bar: PinInitMe<'b, Foo, G>,
) -> InitProof<(), G> {
    init! { bar => Foo {
         .msg = "jhello world".to_owned();
    }}
}

fn main() {
    let bar = Box::pin(MaybeUninit::<Bar2>::uninit());
    init! { bar => Bar2 {
        evil(.bar);
    }};
}
