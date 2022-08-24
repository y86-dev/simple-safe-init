use crate::{InitMe, InitPointer, PinInitMe};
#[cfg(feature = "std")]
use alloc::boxed::Box;
use core::{mem::MaybeUninit, pin::Pin};

/// Central trait to facilitate initialization. Every partially init-able Place should implement this type.
///
/// If you need to implement this type, pay close attention to the comments on the methods of this
/// trait. You need to strictly adhere to the invariants listed, otherwise this library cannot
/// guarantee the soundness of your program.
///
/// # Safety
///
/// This trait requires that partially initialized values of type `Raw` can be stored and initialized
/// values of type `Raw` can be stored by `Init`.
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
    #[doc(hidden)]
    unsafe fn ___init_me<G>(&mut self, guard: G) -> Self::InitMe<'_, G> {
        unsafe { InitPointer::___new(self.___as_mut_ptr(&|_| {}), guard) }
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
    /// # Implementors
    /// To anyone implementing this function, look at the example implementations.
    ///
    /// When this function is being called, the callee can assume that all fields of the `Raw`
    /// pointee have been initialized via access through the raw pointer.
    ///
    /// Some smart pointers have layouts that depend upon the type parameters, take care of the
    /// translation in this function. No other side effects.
    #[doc(hidden)]
    unsafe fn ___init(self) -> Self::Init;

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
    /// # Implementors
    /// To anyone implementing this function, look at the example implementations.
    ///
    /// Calling this function is only marked as unsafe, because it should not be called by normal
    /// code. It should never induce UB or assume any other invariant is upheld.
    /// Always return the same pointer. No side effects.
    #[doc(hidden)]
    unsafe fn ___as_mut_ptr(&mut self, _proof: &impl FnOnce(&Self::Raw)) -> *mut Self::Raw;
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

    unsafe fn ___init(self) -> Self::Init {
        unsafe { self.assume_init() }
    }

    unsafe fn ___as_mut_ptr(&mut self, _proof: &impl FnOnce(&Self::Raw)) -> *mut Self::Raw {
        self.as_mut_ptr()
    }
}

#[cfg(feature = "std")]
unsafe impl<T> PartialInitPlace for Box<MaybeUninit<T>> {
    type Init = Box<T>;
    type Raw = T;
    type InitMe<'a, G>
    = InitMe<'a, T, G>
    where
        Self: 'a
    ;
    unsafe fn ___init(self) -> Self::Init {
        unsafe { self.assume_init() }
    }

    unsafe fn ___as_mut_ptr(&mut self, _proof: &impl FnOnce(&Self::Raw)) -> *mut Self::Raw {
        MaybeUninit::as_mut_ptr(&mut **self)
    }
}

unsafe impl<P, T> PartialInitPlace for Pin<P>
where
    P: PartialInitPlace + core::ops::DerefMut<Target = T>,
    P::Init: core::ops::Deref,
    T: PartialInitPlace<Raw = P::Raw>,
{
    type Init = Pin<P::Init>;
    type Raw = P::Raw;
    type InitMe<'a, G>
    = PinInitMe<'a, P::Raw, G>
    where
        Self: 'a
    ;
    unsafe fn ___init(self) -> Self::Init {
        // TODO is this safe?
        unsafe { Pin::new_unchecked(Pin::into_inner_unchecked(self).___init()) }
    }

    unsafe fn ___as_mut_ptr(&mut self, _proof: &impl FnOnce(&Self::Raw)) -> *mut Self::Raw {
        unsafe { Pin::get_unchecked_mut(self.as_mut()).___as_mut_ptr(_proof) }
    }
}

unsafe impl<P, T> PinnedPlace for Pin<P>
where
    P: PartialInitPlace + core::ops::DerefMut<Target = T>,
    P::Init: core::ops::Deref,
    T: PartialInitPlace<Raw = P::Raw>,
{
}

/// A [PartialInitPlace] that can be allocated without any extra parameters (it is fully
/// uninitialized).
pub trait AllocablePlace: Sized + PartialInitPlace {
    type Error;

    /// Allocate a place of this kind. This might fail.
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

#[derive(Debug)]
pub enum BoxAllocErr<E> {
    Nested(E),
    BoxAlloc,
}

#[cfg(feature = "std")]
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

    unsafe fn ___init(self) -> Self::Init {
        panic!("this function is not designed to be called!")
    }

    unsafe fn ___as_mut_ptr(&mut self, _proof: &impl FnOnce(&Self::Raw)) -> *mut Self::Raw {
        self.inner.as_mut_ptr()
    }
}

/// DO NOT IMPLEMENT MANUALLY, use the `pin_data!` macro.
#[doc(hidden)]
pub unsafe trait ___PinData {
    #[doc(hidden)]
    type ___PinData;
}
