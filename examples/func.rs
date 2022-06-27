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

pub mod nested {
    pub mod foo {
        use easy_init::*;
        pub struct Bar<T>(T);
        impl<T> Bar<T> {
            pub fn baz<G>(val: InitMe<'_, T, G>, v: T) -> InitProof<(), G> {
                val.write(v)
            }
        }
    }
}

fn main() {
    let foo = Box::pin(MaybeUninit::uninit());
    let foo = init! { foo => Foo {
        .a = 32;
        init_usize(.b);
    }};
    println!("{:?}", foo);
    let foo = Box::pin(MaybeUninit::uninit());
    let foo = init! { foo => Foo {
        .a = 32;
        nested::foo::Bar::<usize>::baz(.b, 0);
    }};
    println!("{:?}", foo);
}
