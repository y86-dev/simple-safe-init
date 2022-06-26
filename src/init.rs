use crate::place::*;
use core::marker::PhantomData;

/// A pointer to an Uninitialized `T` there is no pinning guarantee, so the data might be moved
/// after initialization.
pub struct InitMe<'a, T, G> {
    ptr: *mut T,
    _phantom: PhantomData<(&'a mut T, fn(G) -> G)>,
}

impl<'a, T, G> InitMe<'a, T, G> {
    /// Create a new PartialInit
    ///
    /// # Safety
    ///
    /// The caller guarantees that this function is only called from the `init`/`pin_data` macros.
    pub unsafe fn ___new(ptr: *mut T, _guard: G) -> Self {
        Self {
            ptr,
            _phantom: PhantomData,
        }
    }

    /// Unsafely assume that the value is in reality initialized.
    ///
    /// # Safety
    ///
    /// Only call this if you manually ensured the initialization.
    pub unsafe fn assume_init(self) -> InitProof<(), G> {
        InitProof {
            value: (),
            _phantom: PhantomData,
        }
    }

    /// Initialized the contents via a value.
    pub fn write(self, val: T) -> InitProof<(), G> {
        unsafe {
            // SAFETY: We always create InitMe with a valid pointer
            self.ptr.write(val);
        }
        InitProof {
            value: (),
            _phantom: PhantomData,
        }
    }

    /// Gets a raw pointer from this function, the pointee will initially be uninitialized.
    pub unsafe fn as_mut_ptr(&mut self) -> *mut T {
        self.ptr
    }
}

unsafe impl<'a, T, G> ___PlaceInit for InitMe<'a, T, G> {
    type Init = InitProof<(), G>;
    type Raw = T;

    unsafe fn ___init(self) -> Self::Init {
        InitProof {
            value: (),
            _phantom: PhantomData,
        }
    }

    unsafe fn ___as_mut_ptr(&mut self, _proof: impl FnOnce(Self::Raw)) -> *mut Self::Raw {
        self.ptr
    }
}

/// A pointer to an Uninitialized `T` with a pinning guarantee, so the data cannot be moved
/// after initialization, if it is `!Unpin`.
pub struct PinInitMe<'a, T, G> {
    ptr: *mut T,
    _phantom: PhantomData<(&'a mut T, fn(G) -> G)>,
}

impl<'a, T, G> PinInitMe<'a, T, G> {
    /// Create a new PinPartialInit
    ///
    /// # Safety
    /// The caller guarantees that this function is only called by the `init` macro
    pub unsafe fn ___new(ptr: *mut T, _guard: G) -> Self {
        Self {
            ptr,
            _phantom: PhantomData,
        }
    }

    /// Unsafely assume that the value is in reality initialized.
    ///
    /// # Safety
    ///
    /// Only call this if you manually ensured the initialization.
    pub unsafe fn assume_init(self) -> InitProof<(), G> {
        InitProof {
            value: (),
            _phantom: PhantomData,
        }
    }

    /// Initialized the contents via a value.
    pub fn write(self, val: T) -> InitProof<(), G> {
        unsafe {
            self.ptr.write(val);
        }
        InitProof {
            value: (),
            _phantom: PhantomData,
        }
    }

    /// Gets a raw pointer from this function, the pointee will initially be uninitialized.
    pub unsafe fn as_mut_ptr(&mut self) -> *mut T {
        self.___as_mut_ptr(|_x| {})
    }
}

unsafe impl<'a, T, G> ___PlaceInit for PinInitMe<'a, T, G> {
    type Init = InitProof<(), G>;
    type Raw = T;

    unsafe fn ___init(self) -> Self::Init {
        InitProof {
            value: (),
            _phantom: PhantomData,
        }
    }

    unsafe fn ___as_mut_ptr(&mut self, _proof: impl FnOnce(Self::Raw)) -> *mut Self::Raw {
        self.ptr
    }
}

unsafe impl<'a, T, G> ___PinnedPlace for PinInitMe<'a, T, G> {}

/// Proof to show to the `init!` macro, that a value was indeed initialized.
pub struct InitProof<T, G> {
    value: T,
    _phantom: PhantomData<fn(G) -> G>,
}

impl<T, G> InitProof<T, G> {
    /// Unwrap the actual result contained within
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
