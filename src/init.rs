use crate::place::*;
use core::marker::PhantomData;

/// A pointer to an Uninitialized `T` there is no pinning guarantee, so the data might be moved
/// after initialization.
///
/// *Implementation Detail:*
///
/// The second type parameter `G` is a guard type value. It is used to ensure that this object
/// returns a unique `InitProof<(), G>` that cannot be used to vouche for any other initialization
/// except this one.
pub struct InitMe<'a, T: ?Sized, G> {
    ptr: *mut T,
    _phantom: PhantomData<(&'a mut T, fn(G) -> G)>,
}

impl<'a, T: ?Sized, G> InitMe<'a, T, G> {
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
    #[doc(hidden)]
    pub unsafe fn ___new(ptr: *mut T, _guard: G) -> Self {
        Self {
            ptr,
            _phantom: PhantomData,
        }
    }

    /// Unsafely assume that the value is initialized.
    ///
    /// # Safety
    ///
    /// The caller guarantees that the pointee has been fully initialized (e.g. via `as_mut_ptr`).
    ///
    /// *Warning:* This function circumvents the protection created by this library. Try to avoid
    /// using this function.
    pub unsafe fn assume_init(self) -> InitProof<(), G> {
        InitProof {
            value: (),
            _phantom: PhantomData,
        }
    }

    /// Initialized the contents via a value. This overwrites the memory pointed to without
    /// dropping the old pointee.
    pub fn write(self, val: T) -> InitProof<(), G>
    where
        T: Sized,
    {
        unsafe {
            // SAFETY: We always create InitMe with a valid pointer
            self.ptr.write(val);
        }
        InitProof {
            value: (),
            _phantom: PhantomData,
        }
    }

    /// Gets a raw pointer to the pointee. Initially the memory will be uninitialized. If you
    /// initialize parts of the pointee and then call `write`, then those will be overwritten.
    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.ptr
    }
}

#[doc(hidden)]
unsafe impl<'a, T: ?Sized, G> ___PlaceInit for InitMe<'a, T, G> {
    type Init = InitProof<(), G>;
    type Raw = T;
    type InitMe<'b, GG>
    = InitMe<'b, T, G>
    where
        Self: 'b
    ;

    unsafe fn ___init_me<GG>(&mut self, _guard: GG) -> Self::InitMe<'_, G> {
        InitMe {
            ptr: self.ptr,
            _phantom: PhantomData,
        }
    }

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
    unsafe fn ___init(self) -> Self::Init {
        InitProof {
            value: (),
            _phantom: PhantomData,
        }
    }

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
    unsafe fn ___as_mut_ptr(&mut self, _proof: &impl FnOnce(&Self::Raw)) -> *mut Self::Raw {
        self.ptr
    }
}

/// A pointer to an Uninitialized `T` with a pinning guarantee, so the data cannot be moved
/// after initialization, if it is `!Unpin`.
///
/// *Implementation Detail:*
///
/// The second type parameter `G` is a guard type value. It is used to ensure that this object
/// returns a unique `InitProof<(), G>` that cannot be used to vouche for any other initialization
/// except this one.
pub struct PinInitMe<'a, T: ?Sized, G> {
    ptr: *mut T,
    _phantom: PhantomData<(&'a mut T, fn(G) -> G)>,
}

impl<'a, T: ?Sized, G> PinInitMe<'a, T, G> {
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
    #[doc(hidden)]
    pub unsafe fn ___new(ptr: *mut T, _guard: G) -> Self {
        Self {
            ptr,
            _phantom: PhantomData,
        }
    }

    /// Unsafely assume that the value is initialized.
    ///
    /// # Safety
    ///
    /// The caller guarantees that the pointee has been fully initialized (e.g. via `as_mut_ptr`).
    ///
    /// *Warning:* This function circumvents the protection created by this library. Try to avoid
    /// using this function.
    pub unsafe fn assume_init(self) -> InitProof<(), G> {
        InitProof {
            value: (),
            _phantom: PhantomData,
        }
    }

    /// Initialized the contents via a value. This overwrites the memory pointed to without
    /// dropping the old pointee.
    pub fn write(self, val: T) -> InitProof<(), G>
    where
        T: Sized,
    {
        unsafe {
            self.ptr.write(val);
        }
        InitProof {
            value: (),
            _phantom: PhantomData,
        }
    }

    /// Gets a raw pointer to the pointee. Initially the memory will be uninitialized. If you
    /// initialize parts of the pointee and then call `write`, then those will be overwritten.
    ///
    /// The caller is not allowed to move out of the pointer, as it is pinned.
    pub fn as_mut_ptr(&mut self) -> *mut T {
        unsafe { self.___as_mut_ptr(&|_| {}) }
    }
}

#[doc(hidden)]
unsafe impl<'a, T: ?Sized, G> ___PlaceInit for PinInitMe<'a, T, G> {
    type Init = InitProof<(), G>;
    type Raw = T;
    type InitMe<'b, GG>
    = PinInitMe<'b, T, G>
    where
        Self: 'b
    ;

    unsafe fn ___init_me<GG>(&mut self, _guard: GG) -> Self::InitMe<'_, G> {
        PinInitMe {
            ptr: self.ptr,
            _phantom: PhantomData,
        }
    }

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
    unsafe fn ___init(self) -> Self::Init {
        InitProof {
            value: (),
            _phantom: PhantomData,
        }
    }

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
    unsafe fn ___as_mut_ptr(&mut self, _proof: &impl FnOnce(&Self::Raw)) -> *mut Self::Raw {
        self.ptr
    }
}

unsafe impl<'a, T, G> ___PinnedPlace for PinInitMe<'a, T, G> {}

/// Proof to show, that a value was indeed initialized.
///
/// The first parameter `T` is a wrapped value that was the normal return value of the function.
///
/// The second parameter `G` is a guard type value that is set up and used by the macros to ensure
/// sound initialization.
pub struct InitProof<T, G> {
    value: T,
    _phantom: PhantomData<fn(G) -> G>,
}

impl<T, G> InitProof<T, G> {
    /// Unwrap the actual result contained within and validate that the correct guard type was
    /// used.
    pub fn unwrap(self, _guard: G) -> T {
        self.value
    }
}

impl<G> InitProof<(), G> {
    /// Return a value instead of `()`
    pub fn ret<T>(self, value: T) -> InitProof<T, G> {
        InitProof {
            value,
            _phantom: PhantomData,
        }
    }
}
