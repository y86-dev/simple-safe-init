//! Module marking memory locations and pointer usable for (pinned) initialization.
//!
//! Normally it should not be necessary to use types of this module directly. If you need to
//! implement support for a smart pointer however, you need to implement [`PartialInitPlace`] for
//! it. Please read the hidden documentation in the code for each of the functions you need to
//! implement.

use crate::{InitMe, InitPointer, PinInitMe};
#[cfg(feature = "std")]
use alloc::boxed::Box;
use core::{mem::MaybeUninit, pin::Pin};

macro_rules! cfg_std {
    ($($stuff:item)*) => {
        $(
            // TODO change to docsrs
            #[cfg_attr(feature = "docsrs", doc(cfg(feature = "std")))]
            #[cfg(feature = "std")]
            $stuff
        )*
    }
}

/// Central trait to facilitate initialization. Every partially init-able place should implement this type.
///
/// If you need to implement this type, pay close attention to the comments on the methods of this
/// trait. You need to strictly adhere to the invariants listed, otherwise this library cannot
/// guarantee the soundness of your program.
///
/// # Safety
///
/// This trait requires that partially initialized values of type `Raw` can be stored in
/// `Self` (mostly in [`MaybeUninit<T>`]). And initialized values of type `Raw` can be stored by `Init`.
///
/// It only makes sense to implement this type for
/// - direct memory locations (such as [`MaybeUninit<T>`]),
/// - smart pointers with unique access to the pointee.
///
/// When implementing this type you need to be careful, as a faulty implementation can lead to UB
/// at a later point in time.
pub unsafe trait PartialInitPlace {
    /// This is the type `Self` will become, when everything is fully initialized.
    type Init;
    /// This is the actual raw type that needs to be initialized. For smart pointers this is the
    /// pointee type.
    type Raw: ?Sized;
    /// This type should either be `PinInit<'a, Self::Raw, G>` or `InitMe<'a, Self::Raw, G>`.
    /// It is used to completely delegate the initialization to a single function.
    type InitMe<'a, G>: InitPointer<'a, Self::Raw, G>
    where
        Self: 'a;

    /// # **WARNING: MACRO ONLY FUNCTION**
    ///
    /// This function is only designed to be called by the macros of this library.
    /// Using it directly might run into **unexpected and undefined behavior!**
    ///
    /// I repeat: **DO NOT USE THIS FUNCTION!!**
    ///
    /// # Safety
    ///
    /// The caller guarantees that this function is only called from macros of this library.
    ///
    /// # Implementing
    ///
    /// This function should not be implemented manually!
    #[doc(hidden)]
    unsafe fn ___init_me<G>(this: &mut Self, guard: G) -> Self::InitMe<'_, G> {
        // SAFETY: macro only function
        unsafe { InitPointer::___new(Self::___as_mut_ptr(this, &|_| {}), guard) }
    }

    /// # **WARNING: MACRO ONLY FUNCTION**
    ///
    /// This function is only designed to be called by the macros of this library.
    /// Using it directly might run into **unexpected and undefined behavior!**
    ///
    /// I repeat: **DO NOT USE THIS FUNCTION!!**
    ///
    /// # Safety
    ///
    /// The caller guarantees that this function is only called from macros of this library.
    ///
    /// # Implementing
    ///
    /// When this function is being called, the callee can assume that all fields of the `Raw`
    /// pointee have been initialized via access through the raw pointer.
    ///
    /// Some smart pointers have layouts that depend upon the type parameters, take care of the
    /// translation in this function. For example: [`Box<T>`] and [`Box<MaybeUninit<T>>`] **are
    /// not** layout compatible, even though [`MaybeUninit<T>`] and `T` are! For more information
    /// on this, view [this UCG issue](https://github.com/rust-lang/unsafe-code-guidelines/issues/329) (unsafe code guidelines).
    ///
    /// If `Self` is a pointer type, then this function is not allowed to change the memory location
    /// of the initialized memory.
    ///
    /// No side effects allowed.
    ///
    /// [`Box<T>`]: [`alloc::boxed::Box`]
    /// [`Box<MaybeUninit<T>>`]: [`core::mem::MaybeUninit`]
    /// [`MaybeUninit<T>`]: [`core::mem::MaybeUninit`]
    #[doc(hidden)]
    unsafe fn ___init(this: Self) -> Self::Init;

    /// # **WARNING: MACRO ONLY FUNCTION**
    ///
    /// This function is only designed to be called by the macros of this library.
    /// Using it directly might run into **unexpected and undefined behavior!**
    ///
    /// I repeat: **DO NOT USE THIS FUNCTION!!**
    ///
    /// # Safety
    ///
    /// The caller guarantees that this function is only called from macros of this library.
    ///
    /// # Implementing
    ///
    /// - calling this function is only marked as unsafe, because it should not be called by normal
    /// code,
    /// - always return the same pointer,
    /// - no side effects allowed.
    #[doc(hidden)]
    unsafe fn ___as_mut_ptr(this: &mut Self, _proof: &impl FnOnce(&Self::Raw)) -> *mut Self::Raw;

    #[doc(hidden)]
    unsafe fn ___i_have_read_the_documetation_and_verified_that_everything_is_correct();
}

/// Marker trait used to mark Places where the value cannot be moved out of. Example:
/// `Pin<Box<$value>>`.
///
/// # Safety
/// The value at this place cannot be moved.
pub unsafe trait PinnedPlace: PartialInitPlace {}

unsafe impl<T> PartialInitPlace for MaybeUninit<T> {
    type Init = T;
    type Raw = T;
    type InitMe<'a, G>
    = InitMe<'a, T, G>
    where
        Self: 'a
    ;

    unsafe fn ___init(this: Self) -> Self::Init {
        // SAFETY: `T` has been initialized.
        unsafe { this.assume_init() }
    }

    unsafe fn ___as_mut_ptr(this: &mut Self, _proof: &impl FnOnce(&Self::Raw)) -> *mut Self::Raw {
        this.as_mut_ptr()
    }

    unsafe fn ___i_have_read_the_documetation_and_verified_that_everything_is_correct() {}
}

cfg_std! {
    unsafe impl<T> PartialInitPlace for Box<MaybeUninit<T>> {
        type Init = Box<T>;
        type Raw = T;
        type InitMe<'a, G>
        = InitMe<'a, T, G>
        where
            Self: 'a
        ;
        unsafe fn ___init(this: Self) -> Self::Init {
            // SAFETY: `T` has been initialized
            unsafe { this.assume_init() }
        }

        unsafe fn ___as_mut_ptr(this: &mut Self, _proof: &impl FnOnce(&Self::Raw)) -> *mut Self::Raw {
            MaybeUninit::as_mut_ptr(&mut **this)
        }

        unsafe fn ___i_have_read_the_documetation_and_verified_that_everything_is_correct() {}
    }
}

unsafe impl<P, T> PartialInitPlace for Pin<P>
where
    P: PartialInitPlace + core::ops::DerefMut<Target = T>,
    P::Init: core::ops::Deref<Target = P::Raw>,
    T: PartialInitPlace<Raw = P::Raw>,
{
    type Init = Pin<P::Init>;
    type Raw = P::Raw;
    type InitMe<'a, G>
    = PinInitMe<'a, P::Raw, G>
    where
        Self: 'a
    ;
    unsafe fn ___init(this: Self) -> Self::Init {
        // SAFETY: P::___init will not change the address of the pointer, so we can re-pin the
        // returned smart pointer (it is a pointer, because it implements Deref)
        unsafe { Pin::new_unchecked(P::___init(Pin::into_inner_unchecked(this))) }
    }

    unsafe fn ___as_mut_ptr(this: &mut Self, _proof: &impl FnOnce(&Self::Raw)) -> *mut Self::Raw {
        // SAFETY: macro never moves out of the pointer returned
        unsafe { T::___as_mut_ptr(Pin::get_unchecked_mut(this.as_mut()), _proof) }
    }

    unsafe fn ___i_have_read_the_documetation_and_verified_that_everything_is_correct() {}
}

unsafe impl<P, T> PinnedPlace for Pin<P>
where
    P: PartialInitPlace + core::ops::DerefMut<Target = T>,
    P::Init: core::ops::Deref<Target = P::Raw>,
    T: PartialInitPlace<Raw = P::Raw>,
{
}

/// A [PartialInitPlace] that can be allocated without any extra parameters (it is fully
/// uninitialized).
pub trait AllocablePlace: Sized + PartialInitPlace {
    type Error;

    /// Allocate a place of this kind.
    ///
    /// # Errors
    ///
    /// This might fail when not enough memory of the specified kind is available.
    /// If it cannot fail, `Self::Error` should be `!` (the [never type](https://doc.rust-lang.org/reference/types/never.html)).
    fn allocate() -> Result<Self, Self::Error>;
}

impl<T> AllocablePlace for MaybeUninit<T> {
    type Error = !;

    fn allocate() -> Result<Self, Self::Error> {
        Ok(MaybeUninit::uninit())
    }
}

impl<A: AllocablePlace> AllocablePlace for Pin<A>
where
    Pin<A>: From<A> + PartialInitPlace,
{
    type Error = A::Error;

    fn allocate() -> Result<Self, A::Error> {
        Ok(Pin::from(A::allocate()?))
    }
}

cfg_std! {
    /// An allocation error occurring when trying to allocate [`Box<A>`] where `A:` [`AllocablePlace`].
    #[derive(Debug)]
    pub enum BoxAllocErr<E> {
        Nested(E),
        BoxAlloc,
    }

    impl<A: AllocablePlace> AllocablePlace for Box<A>
    where
        Box<A>: PartialInitPlace,
    {
        type Error = BoxAllocErr<A::Error>;

        fn allocate() -> Result<Self, Self::Error> {
            Ok(
                Box::try_new(A::allocate().map_err(|e| BoxAllocErr::Nested(e))?)
                    .map_err(|_| BoxAllocErr::BoxAlloc)?,
            )
        }
    }
}

/// # Safety
/// Only use this type in static fields and initialize the contents before they are being used
/// (deref for example).
pub struct StaticInit<T> {
    inner: MaybeUninit<T>,
}

impl<T> StaticInit<T> {
    /// # Safety
    /// You need to initialize the contents via the init! macro.
    pub const unsafe fn new() -> Self {
        Self {
            inner: MaybeUninit::uninit(),
        }
    }
}

impl<T> core::ops::Deref for StaticInit<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe {
            // SAFETY: our type invariants dictates that our value has been initialized.
            self.inner.assume_init_ref()
        }
    }
}

impl<T> core::ops::DerefMut for StaticInit<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            // SAFETY: our type invariant dictates that our value has been initialized.
            self.inner.assume_init_mut()
        }
    }
}

unsafe impl<T> PartialInitPlace for StaticInit<T> {
    type Init = !;
    type Raw = T;
    type InitMe<'a, G> = PinInitMe<'a, T, G> where Self: 'a;

    unsafe fn ___init(_this: Self) -> Self::Init {
        panic!("this function is not designed to be called!")
    }

    unsafe fn ___as_mut_ptr(this: &mut Self, _proof: &impl FnOnce(&Self::Raw)) -> *mut Self::Raw {
        this.inner.as_mut_ptr()
    }

    unsafe fn ___i_have_read_the_documetation_and_verified_that_everything_is_correct() {}
}

/// DO NOT IMPLEMENT MANUALLY, use the `pin_data!` macro.
#[doc(hidden)]
pub unsafe trait ___PinData {
    #[doc(hidden)]
    type ___PinData;
}
