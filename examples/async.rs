use core::mem::MaybeUninit;
use simple_safe_init::*;

// very inefficient but testing only executor
mod executor {
    use core::{
        future::Future,
        task::{Context, Poll, Waker},
    };
    use std::{sync::Arc, task::Wake};
    pub fn execute<O>(fut: impl Future<Output = O>) -> O {
        let mut fut = Box::pin(fut);
        let w = Waker::from(Arc::new(VoidWaker));
        let mut ctx = Context::from_waker(&w);
        loop {
            println!("Polling future");
            match fut.as_mut().poll(&mut ctx) {
                Poll::Pending => {}
                Poll::Ready(o) => return o,
            }
        }
    }

    struct VoidWaker;

    impl Wake for VoidWaker {
        fn wake(self: Arc<Self>) {
            // do nothing, as we are actively polling...
        }
    }
}

pin_data! {
    #[derive(Debug)]
    struct Foo {
        #pin
        a: u64,
        #pin
        b: u32,
    }
}

struct Yield(bool);

impl core::future::Future for Yield {
    type Output = ();

    fn poll(
        mut self: core::pin::Pin<&mut Self>,
        _: &mut core::task::Context,
    ) -> core::task::Poll<()> {
        if self.0 {
            core::task::Poll::Ready(())
        } else {
            self.0 = true;
            core::task::Poll::Pending
        }
    }
}

async fn init_num<N: From<u8>, G: Guard>(this: PinInitMe<'_, N, G>) -> InitProof<(), G> {
    Yield(false).await;
    this.write(3.into())
}

fn main() {
    executor::execute(async {
        let foo = init! { Box::pin(MaybeUninit::uninit()) => Foo {
            init_num(.a).await;
            println!("a is initialized");
            init_num(.b).await;
        }};
        println!("{:?}", foo);
    });
}
