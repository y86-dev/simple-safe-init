use crate::{init::InitPointer, InitMe, PinInitMe};
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



/// DO NOT IMPLEMENT MANUALLY, use the `pin_data!` macro.
#[doc(hidden)]
pub unsafe trait ___PinData {
    #[doc(hidden)]
    type ___PinData;
}
