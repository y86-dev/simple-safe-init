#![feature(allocator_api, new_uninit, never_type)]

use core::pin::Pin;
use simple_safe_init::*;
use std::alloc::AllocError;

mod buf {
    use super::*;
    use std::alloc::AllocError;

    pin_data! {
        pub struct Buffers {
            big_buf: Box<[u8; 1024 * 1024 * 1024]>,
            #pin
            sml_buf: [u8; 1024],
        }
    }

    impl Buffers {
        pub fn init<G: Guard>(
            this: PinInitMe<'_, Self, G>,
        ) -> Result<InitProof<(), G>, AllocError> {
            Ok(init! { this => Self {
                let buf = Box::try_new_zeroed()?;
                let buf = unsafe {
                    // SAFETY: Buffer has been zeroed
                    buf.assume_init()
                };
                .big_buf = buf;
                .sml_buf = [0; 1024];
            }})
        }

        pub fn big_buf_len(self: Pin<&mut Self>) -> usize {
            self.big_buf.len()
        }
    }
}

use buf::*;

fn main() -> Result<(), AllocError> {
    let buffers: Result<Pin<Box<Buffers>>, AllocError> = init!(@Buffers::init(Pin<Box<Buffers>>)?);
    let mut buffers = buffers.unwrap();
    println!("{}", buffers.as_mut().big_buf_len());
    Ok(())
}
