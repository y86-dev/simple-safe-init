use crate::{InitMe, PinInitMe};
use core::{mem::MaybeUninit, pin::Pin};

/// Central trait to facilitate initialization. Every partially initable Place should implement this type.
///
/// # Safety
///
/// This trait requires that partially initialized values of type `Raw` can be stored and initialiezd
/// values of type `Raw` can be stored by `Init`.
pub unsafe trait ___PlaceInit {
    /// This is the type `Self` will become, when everything is fully initialized.
    type Init;
    /// This is the actual raw type that needs to be initialized.
    type Raw: ?Sized;
    /// This type should either be `PinInit<'a, Self::Raw, G>` or `InitMe<'a, Self::Raw, G>`.
    /// It is used to completely delegate the initialization to a single function.
    type InitMe<'a, G>
    where
        Self: 'a;

    /// # ⛔⛔**WARNING: MACRO ONLY FUNCTION**⛔⛔
    ///
    /// This function is only designed to be used within the macros of this library.
    /// Using it directly might run into **unexpected and undefined behavior!**
    ///
    /// I repeat: **DO NOT USE THIS FUNCTON!!**
    ///
    /// # Safety
    ///
    /// The caller guarantees that this function is only called from macros of this library.
    ///
    /// # Implementors
    /// To anyone implementing this function, look at the example implementations.
    ///
    /// Create a new `Self::InitMe` from the contained pointer. No side effects. Only use guard to
    /// create the `Self::InitMe`.
    #[doc(hidden)]
    unsafe fn ___init_me<G>(&mut self, guard: G) -> Self::InitMe<'_, G>;

    /// # ⛔⛔**WARNING: MACRO ONLY FUNCTION**⛔⛔
    ///
    /// This function is only designed to be used within the macros of this library.
    /// Using it directly might run into **unexpected and undefined behavior!**
    ///
    /// I repeat: **DO NOT USE THIS FUNCTON!!**
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

    /// # ⛔⛔**WARNING: MACRO ONLY FUNCTION**⛔⛔
    ///
    /// This function is only designed to be used within the macros of this library.
    /// Using it directly might run into **unexpected and undefined behavior!**
    ///
    /// I repeat: **DO NOT USE THIS FUNCTON!!**
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
pub unsafe trait ___PinnedPlace {}

unsafe impl<T> ___PlaceInit for MaybeUninit<T> {
    type Init = T;
    type Raw = T;
    type InitMe<'a, G>
    = InitMe<'a, T, G>
    where
        Self: 'a
    ;

    unsafe fn ___init_me<G>(&mut self, guard: G) -> Self::InitMe<'_, G> {
        unsafe { InitMe::___new(self.as_mut_ptr(), guard) }
    }

    unsafe fn ___init(self) -> Self::Init {
        unsafe { self.assume_init() }
    }

    unsafe fn ___as_mut_ptr(&mut self, _proof: &impl FnOnce(&Self::Raw)) -> *mut Self::Raw {
        self.as_mut_ptr()
    }
}

unsafe impl<T> ___PlaceInit for Box<MaybeUninit<T>> {
    type Init = Box<T>;
    type Raw = T;
    type InitMe<'a, G>
    = InitMe<'a, T, G>
    where
        Self: 'a
    ;

    unsafe fn ___init_me<G>(&mut self, guard: G) -> Self::InitMe<'_, G> {
        unsafe { InitMe::___new(self.as_mut_ptr(), guard) }
    }

    unsafe fn ___init(self) -> Self::Init {
        unsafe { self.assume_init() }
    }

    unsafe fn ___as_mut_ptr(&mut self, _proof: &impl FnOnce(&Self::Raw)) -> *mut Self::Raw {
        MaybeUninit::as_mut_ptr(&mut **self)
    }
}

unsafe impl<P, T> ___PlaceInit for Pin<P>
where
    P: ___PlaceInit + core::ops::DerefMut<Target = T>,
    P::Init: core::ops::Deref,
    T: ___PlaceInit<Raw = P::Raw>,
{
    type Init = Pin<P::Init>;
    type Raw = P::Raw;
    type InitMe<'a, G>
    = PinInitMe<'a, P::Raw, G>
    where
        Self: 'a
    ;

    unsafe fn ___init_me<G>(&mut self, guard: G) -> Self::InitMe<'_, G> {
        unsafe { PinInitMe::___new(self.___as_mut_ptr(&|_| {}), guard) }
    }

    unsafe fn ___init(self) -> Self::Init {
        unsafe { Pin::new_unchecked(Pin::into_inner_unchecked(self).___init()) }
    }

    unsafe fn ___as_mut_ptr(&mut self, _proof: &impl FnOnce(&Self::Raw)) -> *mut Self::Raw {
        unsafe { Pin::get_unchecked_mut(self.as_mut()).___as_mut_ptr(_proof) }
    }
}

unsafe impl<P, T> ___PinnedPlace for Pin<P>
where
    P: ___PlaceInit + core::ops::DerefMut<Target = T>,
    P::Init: core::ops::Deref,
    T: ___PlaceInit<Raw = P::Raw>,
{
}

#[doc(hidden)]
pub unsafe trait ___PinData {
    #[doc(hidden)]
    type ___PinData;
}
