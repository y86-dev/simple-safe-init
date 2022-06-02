use crate::place::*;
use core::marker::PhantomData;

/// A pointer to an Uninitialized `T` there is no pinning guarantee, so the data might be moved
/// after initialization.
pub struct InitMe<'a, T> {
    ptr: *mut T,
    _phantom: PhantomData<&'a mut T>,
}

impl<'a, T> InitMe<'a, T> {
    /// Create a new PartialInit
    ///
    /// # Safety
    ///
    /// The caller guarantees that this function is only called from the `init`/`pin_data` macros.
    pub unsafe fn ___new(ptr: *mut T) -> Self {
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
    pub unsafe fn assume_init(self) -> InitProof<()> {
        InitProof { value: () }
    }

    /// Initialized the contents via a value.
    pub fn write(self, val: T) -> InitProof<()> {
        unsafe {
            // SAFETY: We always create InitMe with a valid pointer
            self.ptr.write(val);
        }
        InitProof { value: () }
    }

    /// Gets a raw pointer from this function, the pointee will initially be uninitialized.
    pub unsafe fn as_mut_ptr(&mut self) -> *mut T {
        self.ptr
    }
}

unsafe impl<'a, T> ___PlaceInit for InitMe<'a, T> {
    type Init = InitProof<()>;
    type Raw = T;

    unsafe fn ___init(self) -> Self::Init {
        InitProof { value: () }
    }

    unsafe fn ___as_mut_ptr(&mut self, _proof: impl FnOnce(Self::Raw)) -> *mut Self::Raw {
        self.ptr
    }
}

/// A pointer to an Uninitialized `T` with a pinning guarantee, so the data cannot be moved
/// after initialization, if it is `!Unpin`.
pub struct PinInitMe<'a, T> {
    ptr: *mut T,
    _phantom: PhantomData<&'a mut T>,
}

impl<'a, T> PinInitMe<'a, T> {
    /// Create a new PinPartialInit
    ///
    /// # Safety
    /// The caller guarantees that this function is only called by the `init` macro
    pub unsafe fn ___new(ptr: *mut T) -> Self {
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
    pub unsafe fn assume_init(self) -> InitProof<()> {
        InitProof { value: () }
    }

    /// Initialized the contents via a value.
    pub fn write(self, val: T) -> InitProof<()> {
        unsafe {
            self.ptr.write(val);
        }
        InitProof { value: () }
    }

    /// Gets a raw pointer from this function, the pointee will initially be uninitialized.
    pub unsafe fn as_mut_ptr(&mut self) -> *mut T {
        self.___as_mut_ptr(|_x| {})
    }
}

unsafe impl<'a, T> ___PlaceInit for PinInitMe<'a, T> {
    type Init = InitProof<()>;
    type Raw = T;

    unsafe fn ___init(self) -> Self::Init {
        InitProof { value: () }
    }

    unsafe fn ___as_mut_ptr(&mut self, _proof: impl FnOnce(Self::Raw)) -> *mut Self::Raw {
        self.ptr
    }
}

unsafe impl<'a, T> ___PinnedPlace for PinInitMe<'a, T> {}

/// Proof to show to the `init!` macro, that a value was indeed initialized.
pub struct InitProof<T> {
    value: T,
}

impl<T> InitProof<T> {
    /// Unwrap the actual result contained within
    pub fn unwrap(self) -> T {
        self.value
    }
}

impl InitProof<()> {
    /// Return a value instead of `()`
    pub fn ret<T>(self, value: T) -> InitProof<T> {
        InitProof { value }
    }
}
