//! Unique versions of smart pointers from the alloc crate.
//!
//! Inspired by [pin-init](https://docs.rs/pin-init/0.2.0/pin_init/index.html) and [servo_arc](https://docs.rs/servo_arc/latest/servo_arc/struct.UniqueArc.html).

use super::{
    place::{AllocablePlace, PartialInitPlace},
    Guard, InitMe,
};
use alloc::{alloc::AllocError, rc::Rc, sync::Arc};
use core::{
    fmt,
    mem::MaybeUninit,
    ops::{Deref, DerefMut},
    pin::Pin,
};

macro_rules! make_unique {
    ($(#[$attr:meta])* $name:ident, $orig:ident ) => {
        $(#[$attr])*
        #[derive(Debug, Ord, Hash, PartialOrd, Eq, PartialEq)]
        pub struct $name<T: ?Sized> {
            inner: $orig<T>,
        }

        impl<T: ?Sized + fmt::Display> fmt::Display for $name<T> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.inner)
            }
        }

        impl<T> $name<T> {
            #[doc = concat!("Constructs a new [`", stringify!($name), "<T>`].")]
            pub fn new(data: T) -> Self {
                Self { inner: $orig::new(data) }
            }

            #[doc = concat!("Constructs a new [`Pin`]`<`[`", stringify!($name), "<T>`]`>`.")]
            pub fn pin(data: T) -> Pin<Self> {
                // SAFETY: we will be pinned indefinetly.
                unsafe { Pin::new_unchecked(Self { inner: $orig::new(data) }) }
            }

            #[doc = concat!("Constructs a new [`", stringify!($name), "<T>`], returning an error if allocation fails.")]
            pub fn try_new(data: T) -> Result<Self, AllocError> {
                $orig::try_new(data).map(|inner| Self { inner })
            }

            #[doc = concat!("Constructs a new [`", stringify!($name), "<T>`], returning an error if allocation fails.")]
            pub fn try_pin(data: T) -> Result<Pin<Self>, AllocError> {
                // SAFETY: we will be pinned indefinetly.
                Self::try_new(data).map(|s| unsafe { Pin::new_unchecked(s) })
            }
        }

        impl<T: ?Sized> $name<T> {
            #[doc = concat!("Convert to a sharable [`", stringify!($orig), "<T>`].")]
            pub fn share(this: Self) -> $orig<T> {
                this.inner
            }

            #[doc = concat!("Convert to a sharable [`", stringify!($orig), "<T>`].")]
            pub fn pin_share(this: Pin<Self>) -> Pin<$orig<T>> {
                // SAFETY: we do not move out of the pinned pointer.
                unsafe { Pin::new_unchecked(Pin::into_inner_unchecked(this).inner) }
            }

            /// Provides a raw pointer to the data.
            #[doc = concat!("The counts are not affected in any way and the [`", stringify!($orig), "<T>`] is not consumed.")]
            #[doc = concat!("The pointer is valid for as long as there are strong counts in the [`", stringify!($orig), "<T>`].")]
            pub fn as_ptr(this: &Self) -> *const T {
                $orig::as_ptr(&this.inner)
            }
        }

        impl<T: ?Sized> Deref for $name<T> {
            type Target = T;

            fn deref(&self) -> &Self::Target {
                &*self.inner
            }
        }

        impl<T: ?Sized> DerefMut for $name<T> {
            fn deref_mut(&mut self) -> &mut Self::Target {
                // SAFETY: No other arcs exist that point to the same place.
                unsafe { $orig::get_mut_unchecked(&mut self.inner) }
            }
        }

        unsafe impl<T> PartialInitPlace for $name<MaybeUninit<T>> {
            type Init = $name<T>;
            type Raw = T;
            type InitMe<'a, G: Guard> = InitMe<'a, T, G> where Self: 'a;

            unsafe fn ___init(this: Self) -> Self::Init {
                $name {
                    // SAFETY: `T` has been initialized
                    inner: unsafe { $orig::<MaybeUninit<T>>::assume_init(this.inner) },
                }
            }

            unsafe fn ___as_mut_ptr(
                this: &mut Self,
                _proof: &impl FnOnce(&Self::Raw),
            ) -> *mut Self::Raw {
                (**this).as_mut_ptr()
            }
        }

        impl<T> AllocablePlace for $name<T> {
            type Error = AllocError;
            type Alloced = $name<MaybeUninit<T>>;
            type Final = $name<T>;

            fn allocate() -> Result<Self::Alloced, Self::Error> {
                $name::try_new(MaybeUninit::uninit())
            }

            fn after_init(alloced: <Self::Alloced as PartialInitPlace>::Init) -> Self::Final {
                alloced
            }
        }

        impl<T> AllocablePlace for $orig<T> {
            type Error = AllocError;
            type Alloced = $name<MaybeUninit<T>>;
            type Final = $orig<T>;

            fn allocate() -> Result<Self::Alloced, Self::Error> {
                $name::try_new(MaybeUninit::uninit())
            }

            fn after_init(alloced: <Self::Alloced as PartialInitPlace>::Init) -> Self::Final {
                $name::share(alloced)
            }
        }
    };
}

make_unique! {
    /// [`Arc<T>`] but with reference count equal to 1.
    UniqueArc, Arc
}

make_unique! {
    /// [`Rc<T>`] but with reference count equal to 1.
    UniqueRc, Rc
}
