//! Library to safely initialize structs.
//!
//! # Getting Started
//! Suppose you have a struct that you want to initialize while it is pinned. For Example:
//! ```rust
//! use core::{mem::MaybeUninit, pin::Pin, marker::PhantomPinned};
//! use simple_safe_init::*;
//!
//! struct SelfReferentialStruct {
//!     msg: String,
//!     // this will be our field that depends upon the pinning
//!     my_addr: *const SelfReferentialStruct,
//!     _p: PhantomPinned,
//! }
//!
//! impl SelfReferentialStruct {
//!     // a method that only works, if we are pinned
//!     pub fn print_info(self: Pin<&mut Self>) {
//!         println!("'{}' says SelfReferentialStruct at {:p}", self.msg, self.my_addr);
//!     }
//! }
//! ```
//!
//! In order to initialize this struct, we need the address of the struct itself! But we only can
//! have the address, if we have pinned the struct. Thus we need to first pin an uninitialized
//! version of the struct and then initialize it:
//! ```rust
//! # use core::{mem::MaybeUninit, pin::Pin, marker::PhantomPinned};
//! # use simple_safe_init::*;
//!
//! # struct SelfReferentialStruct {
//! #     msg: String,
//! #     // this will be our field that depends upon the pinning
//! #     my_addr: *const SelfReferentialStruct,
//! #     _p: PhantomPinned,
//! # }
//! #
//! # impl SelfReferentialStruct {
//! #     // a method that only works, if we are pinned
//! #     pub fn print_info(self: Pin<&mut Self>) {
//! #         println!("'{}' says SelfReferentialStruct at {:p}", self.msg, self.my_addr);
//! #     }
//! # }
//! // first we need some uninitialized memory, use `core::mem::MaybeUninit` for that:
//! let my_struct = Box::pin(MaybeUninit::uninit());
//! // `init!` consumes its input, so we need to retrive the pointer here
//! let addr = my_struct.as_ptr();
//! let mut my_struct = init! { my_struct => SelfReferentialStruct {
//!     .msg = "Hello World".to_owned();
//!     .my_addr = addr;
//!     ._p = PhantomPinned;
//! }};
//! my_struct.as_mut().print_info();
//! ```
//! All of this without unsafe code and guarantees that you have not forgotten anything. A compile
//! error is emitted, if
//! - a field is missing.
//! - a field is initialized twice.
//!
//!
//!
//! ## What about encapsulation?
//!
//! When you want your struct fields to remain private, but you still need pinned initialization,
//! then you can delegate the initialization to a custom init function:
//! ```rust
//! use core::{mem::MaybeUninit, pin::Pin, marker::PhantomPinned};
//! use simple_safe_init::*;
//!
//! mod structs {
//!     use core::{mem::MaybeUninit, pin::Pin, marker::PhantomPinned};
//!     use simple_safe_init::*;
//!
//!
//!     pub struct MyPinnedStruct {
//!         msg: String,
//!         // this will be our field that depends upon the pinning
//!         my_addr: usize,
//!         _p: PhantomPinned,
//!     }
//!
//!     impl MyPinnedStruct {
//!         // a method that only works, if we are pinned
//!         pub fn print_info(self: Pin<&mut Self>) {
//!             println!("'{}' says MyPinnedStruct at {:X}", self.msg, self.my_addr);
//!         }
//!
//!         // this is an init function, it takes a `PinInitMe` as its first
//!         // argument and returns an `InitProof` verifying the initialization.
//!         // The generic argument `G` is called the guard type, it is needed to ensure soundness
//!         // of the library.
//!         // you can have any number of additional arguments
//!         pub fn init<G>(mut this: PinInitMe<'_, Self, G>, msg: String) -> InitProof<(), G> {
//!             // we still need the address for this example
//!             let addr = this.as_mut_ptr() as usize;
//!             // we still use the same syntax here!
//!             init! { this => Self {
//!                 ._p = PhantomPinned;
//!                 .msg = msg;
//!                 .my_addr = addr;
//!             }}
//!         }
//!     }
//! }
//!
//! // in main:
//! // first we need some uninitialized memory, use `core::mem::MaybeUninit` for that:
//! let my_struct = Box::pin(MaybeUninit::uninit());
//! // now we cannot use the code from before, because the fields of the struct are private...
//! // but we declared the init function earlier, so we just use that:
//! let mut my_struct = init!(structs::MyPinnedStruct::init(my_struct, "Hello World".to_owned()));
//! my_struct.as_mut().print_info();
//! ```
//! See [guard-parameter] to understand why he type parameter is needed.
//!
//!
//! ## Nested types
//!
//! When you are using more complex types, initializing nested types is also necessary. You can use
//! the `pin_data!` macro to define which fields are structurally pinned.
//! ```rust
//! use simple_safe_init::*;
//! use core::{marker::PhantomPinned, mem::MaybeUninit};
//!
//! struct NamedCounter {
//!     msg: String,
//!     count: usize,
//!     // for some reason this type needs pinning...
//!     _p: PhantomPinned,
//! }
//!
//! impl NamedCounter {
//!     pub fn init<G>(this: PinInitMe<'_, Self, G>, msg: String) -> InitProof<(), G> {
//!         init! { this => Self {
//!             .msg = msg;
//!             .count = 0;
//!             ._p = PhantomPinned;
//!         }}
//!     }
//! }
//!
//! // we need to tell the macro which fields are structurally pinned
//! pin_data! {
//!     struct Bar {
//!         #pin
//!         first: NamedCounter,
//!         #pin
//!         second: NamedCounter,
//!     }
//! }
//!
//! let bar = Box::pin(MaybeUninit::uninit());
//! let bar = init! { bar => Bar {
//!     // you can use the init functions like this:
//!     // only the first argument can be a field though.
//!     NamedCounter::init(.first, "First".to_owned());
//!     NamedCounter::init(.second, "Second".to_owned());
//! }};
//! ```
//!
//! ## Macro initialization
//!
//! If you have defined some macros which can initialize values, then you can use them like this:
//! ```rust
//! use core::{mem::MaybeUninit, pin::Pin, marker::PhantomPinned};
//! use simple_safe_init::*;
//!
//! // we also need to tell the macro what fields are structurally pinned.
//! pin_data! {
//!     struct MyPinnedStruct {
//!         msg: String,
//!         // this will be our field that depends upon the pinning
//!         my_addr: usize,
//!         _p: PhantomPinned,
//!     }
//! }
//!
//! impl MyPinnedStruct {
//!     // a method that only works, if we are pinned
//!     pub fn print_info(self: Pin<&mut Self>) {
//!         println!("'{}' says MyPinnedStruct at {:X}", self.msg, self.my_addr);
//!     }
//! }
//!
//! // init macro
//! macro_rules! init_addr {
//!     ($addr:expr, $val:expr) => {
//!         $addr.write($val)
//!         // need to return expression evaluating to InitProof
//!     }
//! }
//!
//! // in main:
//! // first we need some uninitialized memory, use `core::mem::MaybeUninit` for that:
//! let my_struct = Box::pin(MaybeUninit::uninit());
//! // for this example we need the address...
//! let addr = my_struct.as_ptr() as usize;
//! let mut my_struct = init! { my_struct => MyPinnedStruct {
//!     // same syntax as function calls
//!     init_addr!(.my_addr, addr);
//!     ._p = PhantomPinned;
//!     .msg = "Hello World".to_owned();
//! }};
//! my_struct.as_mut().print_info();
//! ```
//!
//! # Smart Pointer Support
//! See [`PartialInitPlace`].
//!
//!
//!
//! # In Depth Explanations
//!
//! ## Guard Parameter
//!
//! If there were no guard parameter, then it would be possible to vouch for initializing something
//! by providing an InitProof generated by initializing something else.
//!
//! Because of this parameter it is only possible to vouch for initializing the thing connected
//! with that guard parameter. It is essential that a guard parameter is only used once, the macros
//! provided by this library always follow this invariant.
//!
//! ## How does `init!` work?
//!

#![feature(new_uninit, generic_associated_types, never_type, allocator_api)]
#![deny(unsafe_op_in_unsafe_fn)]

mod macros;
pub mod place;

mod tests;

use crate::place::*;
use core::marker::PhantomData;

mod sealed {
    use super::*;
    pub trait Sealed {}

    impl<'a, T: ?Sized, G> Sealed for InitMe<'a, T, G> {}
    impl<'a, T: ?Sized, G> Sealed for PinInitMe<'a, T, G> {}
}

pub trait InitPointer<'a, T: ?Sized, G>: sealed::Sealed {
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
    unsafe fn ___new(ptr: *mut T, guard: G) -> Self;
}

/// A pointer to an Uninitialized `T` there is no pinning guarantee, so the data might be moved
/// after initialization.
///
/// *Implementation Detail:*
///
/// The second type parameter `G` is a guard type value. It is used to ensure that this object
/// returns a unique `InitProof<(), G>` that cannot be used to vouch for any other initialization
/// except this one.
pub struct InitMe<'a, T: ?Sized, G> {
    ptr: *mut T,
    _phantom: PhantomData<(&'a mut T, fn(G) -> G)>,
}

impl<'a, T: ?Sized, G> InitPointer<'a, T, G> for InitMe<'a, T, G> {
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
    unsafe fn ___new(ptr: *mut T, _guard: G) -> Self {
        Self {
            ptr,
            _phantom: PhantomData,
        }
    }
}

impl<'a, T: ?Sized, G> InitMe<'a, T, G> {
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
unsafe impl<'a, T: ?Sized, G> PartialInitPlace for InitMe<'a, T, G> {
    type Init = InitProof<(), G>;
    type Raw = T;
    type InitMe<'b, GG>
    = InitMe<'b, T, GG>
    where
        Self: 'b
    ;

    unsafe fn ___init_me<GG>(&mut self, _guard: GG) -> Self::InitMe<'_, GG> {
        InitMe {
            ptr: self.ptr,
            _phantom: PhantomData,
        }
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
    unsafe fn ___init(self) -> Self::Init {
        InitProof {
            value: (),
            _phantom: PhantomData,
        }
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
/// returns a unique `InitProof<(), G>` that cannot be used to vouch for any other initialization
/// except this one.
pub struct PinInitMe<'a, T: ?Sized, G> {
    ptr: *mut T,
    _phantom: PhantomData<(&'a mut T, fn(G) -> G)>,
}

impl<'a, T: ?Sized, G> InitPointer<'a, T, G> for PinInitMe<'a, T, G> {
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
    unsafe fn ___new(ptr: *mut T, _guard: G) -> Self {
        Self {
            ptr,
            _phantom: PhantomData,
        }
    }
}

impl<'a, T: ?Sized, G> PinInitMe<'a, T, G> {
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
unsafe impl<'a, T: ?Sized, G> PartialInitPlace for PinInitMe<'a, T, G> {
    type Init = InitProof<(), G>;
    type Raw = T;
    type InitMe<'b, GG>
    = PinInitMe<'b, T, GG>
    where
        Self: 'b
    ;

    unsafe fn ___init_me<GG>(&mut self, _guard: GG) -> Self::InitMe<'_, GG> {
        PinInitMe {
            ptr: self.ptr,
            _phantom: PhantomData,
        }
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
    unsafe fn ___init(self) -> Self::Init {
        InitProof {
            value: (),
            _phantom: PhantomData,
        }
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
    unsafe fn ___as_mut_ptr(&mut self, _proof: &impl FnOnce(&Self::Raw)) -> *mut Self::Raw {
        self.ptr
    }
}

unsafe impl<'a, T: ?Sized, G> PinnedPlace for PinInitMe<'a, T, G> {}

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

/// panics.
/// Workaround to avoid a clippy error lint.
pub fn conjure<T>() -> T {
    panic!("this function is not designed to be called!")
}
