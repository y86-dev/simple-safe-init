//! Library to safely initialize pinned structs.
//!
//! This library uses a delcarative macro to make initializing structs safe and more ergonomic.
//!
//! Readers are expected to know what [pinning](https://doc.rust-lang.org/std/pin/index.html) is.
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
//! In order to initialize this struct, we need the address of the struct itself! But we only can
//! have the address, if we have pinned the struct. Thus we need to first pin an uninitialized
//! version of the struct and then initialize it:
//! ```rust
//! # use core::{mem::MaybeUninit, pin::Pin, marker::PhantomPinned};
//! # use simple_safe_init::*;
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
//! let my_struct = Box::pin(MaybeUninit::uninit());
//! // [`init!`] consumes its input, so we need to retrive the pointer here
//! let addr = my_struct.as_ptr();
//! let mut my_struct = init! { my_struct => SelfReferentialStruct {
//!     .msg = "Hello World".to_owned();
//!     .my_addr = addr;
//!     ._p = PhantomPinned;
//! }};
//! my_struct.as_mut().print_info();
//! ```
//! The [`init!`] macro takes the value you want to initialize, its type and an initializer.
//! Within the initializer you can use arbitrary rust statements. To initialize there are a couple
//! of special statements with custom syntax. One of them is: `.$field = $expr;` it initializes the field
//! with the given expression. See [here](#custom-syntax-list) for a complete list of the custom syntax.
//!
//! All of this without unsafe code and guarantees that you have not forgotten anything. A compile
//! error is emitted, if
//! - a field is missing,
//! - a field is initialized multiple times.
//!
//!
//! ## What about encapsulation?
//! The macro relies on the caller having access to all of the structs fields.
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
//!         //
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
//! use structs::MyPinnedStruct;
//!
//! let my_struct = Box::pin(MaybeUninit::uninit());
//! // now we cannot use the code from before, because the fields of the struct are private...
//! // but we declared the init function earlier, so we just use that:
//! let mut my_struct = init!(MyPinnedStruct::init(my_struct, "Hello World".to_owned()));
//! my_struct.as_mut().print_info();
//! ```
//! See [here](#guard-parameter) to understand why the type parameter is needed.
//!
//! When using [`init!`] with an init-function, then you can only use a single init-function, because
//! it already fully initializes the struct. Just supply the allocated uninitialized memory for the
//! struct as the first parameter.
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
//! // we need to tell the macro which fields are structurally pinned.
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
//! `pin_data!` informs the [`init!`] macro what fields are structually pinned by scanning for a
//! `#pin` before any attributes (remember that doc comments are also attributes).
//! The [`init!`] macro creates an init-pointer from the given fields. Depending on the prescense of
//! `#pin` it creates [`InitMe`] or [`PinInitMe`].
//!
//! ## Macro initialization
//!
//! You can also use init-macros, they have a similar syntax as the init-functions. The first thing
//! is an expression evaluating to [`InitMe`] or [`PinInitMe`]. After that a comma is expected, if
//! more input is passed to the macro. No further restrictions exist.
//!
//! Here is an example of an init-macro:
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
//!     ($addr:expr, $val:expr) => {{
//!         let addr = $addr;
//!         let val = $val;
//!         println!("initializing {addr:p} with {val:?}");
//!         addr.write(val)
//!         // need to return expression evaluating to InitProof
//!     }}
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
//! ## Convenient shortcuts
//! There are some shortcuts for common expressions:
//! ### Avoid creating [`MaybeUninit`]
//! In the previous examples, we always had to create some uninitialized memory. It is very common
//! to write `Box::pin(MaybeUninit::uninit())` or doing this with other smart pointers. For that
//! reason the [`init!`] macro supports the following shortcut:
//! ```rust
//! use core::{mem::MaybeUninit, pin::Pin, marker::PhantomPinned};
//! use simple_safe_init::*;
//! struct MyPinnedStruct {
//!     msg: String,
//!     _p: PhantomPinned,
//! }
//!
//! impl MyPinnedStruct {
//!     // a method that only works, if we are pinned
//!     pub fn print_info(self: Pin<&mut Self>) {
//!         println!("Hello from pinned: {}", self.msg);
//!     }
//! }
//! let mut my_struct = init! { @Pin<Box<MaybeUninit<MyPinnedStruct>>> => MyPinnedStruct {
//!     .msg = "Hello World".to_owned();
//!     ._p = PhantomPinned;
//! }};
//! my_struct.as_mut().print_info();
//! ```
//!
//! # Advanced Topics
//! ## Custom syntax list
//! There are two main ways of initializing with [`init!`]:
//! ### Manual initialization
//! This way you need to have access to all sturct fields and you will need to provide an
//! initializer handling every field of the struct induvidually.
//!
//! The initializer allows the following custom syntax while initializing the given `field`:
//! - `.$field = $expr;` where `expr` is any rust expression,
//! - `$func(.$field, $($param),*);` where `func` is an init function with the correct type for
//! `field` (pay attention to the right pin status) and `param` are arbitary rust experssions,
//! - `~let $pat = $func(.$field, $($param),*);` where `func` and `param` are the same as before
//! and `pat` is any rust pattern,
//! - `unsafe { $func(.$field, $($param),*) };` where `func` and `param` as before, except `func`
//! can be `unsafe`,
//! - `~let $pat = unsafe { $func(.$field, $($param),*) };` where `func` and `param` are the same as before
//! and `pat` is any rust pattern,
//! - `$func(.$field, $($param),*).await;` where `func` is an async init function with the correct type for
//! `field` (pay attention to the right pin status) and `param` are arbitary rust experssions,
//! - `~let $pat = $func(.$field, $($param),*).await;` where `func` and `param` are the same as before
//! and `pat` is any rust pattern,
//!
//!
//! ### Single init function/macro
//! This way you will provide a single function or macro initializing the whole struct at once. For
//! this to work you do not need to have access to the fields.
//! ###
//!
//! In both cases you can use `@$type` instead of an expression having that type. It needs to
//! implement the [`AllocablePlace`] trait which dictates how it is allocated.
//!
//!
//! ## Smart Pointer Support
//! See [`PartialInitPlace`].
//!
//!
//!
//! ## Guard Parameter
//!
//! If there were no guard parameter, then it would be possible to vouch for initializing something
//! by providing an InitProof generated by initializing something else.
//!
//! Because of this parameter it is only possible to vouch for initializing the `(Pin)InitMe` connected
//! with that guard parameter. It is essential that a guard parameter is only used once, the macros
//! provided by this library always follow this invariant.
//!
//! ## How does [`init!`] work?
//! This section is intended for readers trying to understand the inner workings of this libarary.
//! If you only intend to use the library you do not need to read this section.
//!
//! The [`init!`] macro uses a combination of `unsafe`, special traits and a struct initializer to
//! ensure safe initialization:
//! ### Special Traits
//! - [`PartialInitPlace`] marks types that can be used as memory locations for initialization,
//! - [`PinnedPlace`] marks [`PartialInitPlace`]s which have stable addresses for the duration of
//! their existence,
//! - [`AllocablePlace`] marks [`PartialInitPlace`]s which can be allocated,
//! - *(hidden)* `___PinData` is implemented by the `pin_data!` macro, it is used to uphold
//! the correct pinning invariants for each of the fields.
//!
//! These traits are mostly used to ensure only the right types are used to house uninitialized
//! values. For example, [`Box<T>`] cannot hold uninitialized values of type `T`. And
//! `Box<MaybeUninit<T>>` cannot be used for a type that requires pinning.
//! ### Unsafe
//! To initialize uninitialized memory one either writes it using [`MaybeUninit::write`] or using
//! raw pointers. The latter of course requires unsafe.
//!
//! When a user writes `.$field = $expr;` in the initializer, [`init!`] creates a raw pointer to
//! `field` and uses [`core::ptr::write`] to set it to `expr`.
//!
//! When a user writes `$func(.$field);`, then a raw pointer is again created and used to create a
//! [`InitMe`] or [`PinInitMe`]. To do this a guard parameter is also required. It is currently
//! implemented as a local type which is shadowed to prevent accidental/malicous use.
//!
//! [`MaybeUninit`]: [`core::mem::MaybeUninit`]
//! [`MaybeUninit::write`]: [`core::mem::MaybeUninit::write`]
//! [`Box<T>`]: [`alloc::boxed::Box<T>`]

#![no_std]
#![cfg_attr(feature = "std", feature(new_uninit))]
#![cfg_attr(feature = "std", feature(allocator_api))]
#![feature(generic_associated_types, never_type)]
#![deny(unsafe_op_in_unsafe_fn)]
#[cfg(feature = "std")]
extern crate alloc;

mod macros;
pub mod place;

#[cfg(tests)]
mod tests;

use crate::place::*;
use core::{
    fmt::{self, Formatter, Pointer},
    marker::PhantomData,
};

mod sealed {
    use super::*;
    pub trait Sealed {}

    impl<'a, T: ?Sized, G> Sealed for InitMe<'a, T, G> {}
    impl<'a, T: ?Sized, G> Sealed for PinInitMe<'a, T, G> {}
}

/// A sealed trait used to enforce the correct function is called and no ambiguity arises when
/// creating [`InitPointer`]s.
///
/// The only types implementing this trait are:
/// - [`InitMe`],
/// - [`PinInitMe`].
pub trait InitPointer<'a, T: ?Sized, G>: sealed::Sealed + Pointer {
    /// # **WARNING: MACRO ONLY FUNCTION**
    ///
    /// This function is only designed to be called by the macros of this library.
    /// Using it directly might run into **unexpected and undefined behavior!**
    ///
    /// I repeat: **DO NOT USE THIS FUNCTION!!**
    ///
    /// # Safety
    /// The caller guarantees:
    /// - this function is only called from macros of this library,
    /// - `ptr` is aligned and pointing to allocated memory,
    /// - `guard` is not accesible by unauthorized code.
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
/// See [here](#guard-parameter) for an explanation on this parameter.
pub struct InitMe<'a, T: ?Sized, G> {
    ptr: *mut T,
    // We need the correct variance, so we only accept the exact type for `G`. `T` and `'a` should
    // behave like `&'a mut T`.
    _phantom: PhantomData<(&'a mut T, fn(G) -> G)>,
}

impl<'a, T: ?Sized, G> Pointer for InitMe<'a, T, G> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:p}", self.ptr)
    }
}

impl<'a, T: ?Sized, G> InitPointer<'a, T, G> for InitMe<'a, T, G> {
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
    /// **Warning:** Careless usage of this function could result in compromising the protection
    /// created by this library. **Try to avoid using this function.**
    ///
    /// # Examples
    ///
    /// ```rust
    /// use core::mem::MaybeUninit;
    /// use simple_safe_init::*;
    /// #[derive(Debug)]
    /// struct Count {
    ///     count: usize,
    /// }
    ///
    /// fn init_count<G>(this: InitMe<'_, Count, G>) -> InitProof<(), G> {
    ///     // SAFETY: We write to uninitialized memory using a raw pointer that is valid
    ///     unsafe { addr_of_mut!((*this.as_mut_ptr()).count).write(42); }
    ///     // SAFETY: We initialized all fields before
    ///     unsafe { this.assume_init() }
    /// }
    ///
    /// let count = Box::new(MaybeUninit::uninit());
    /// let count = init!(init_count(count));
    /// println!("{count:?}");
    /// ```
    pub unsafe fn assume_init(self) -> InitProof<(), G> {
        InitProof {
            value: (),
            _phantom: PhantomData,
        }
    }

    /// Initialized the contents with the given value.
    ///
    /// This overwrites the memory pointed to without dropping the old value.
    ///
    /// # Examples
    /// ```rust
    /// use core::mem::MaybeUninit;
    /// use simple_safe_init::*;
    ///
    /// pin_data! {
    ///     #[derive(Debug)]
    ///     struct Count {
    ///         inner: usize,
    ///     }
    /// }
    ///
    /// let count = Box::new(MaybeUninit::uninit());
    /// let count = init! { count => Count {
    ///     InitMe::write(.inner, 42);
    /// }};
    /// println!("{count:?}");
    /// ```
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

    /// Gets a raw pointer to the pointee.
    ///
    /// Initially (after creation of an [`InitMe`]) the memory will be uninitialized. Because
    /// [`InitMe`] does not track partial initialization, using this function requires great care.
    /// Here are some of the hazards one could encounter:
    /// - overwriting a partially initialized value by calling [`InitMe::write`] (this will
    /// overwrite without calling drop),
    /// - calling [`InitMe::assume_init`] before the value is fully initialized (this is UB)
    ///
    /// This function is specifically designed to be used for:
    /// - careful manual initialization where init! is not sufficient (please check if this is
    /// really necessary),
    /// - getting access to the address of the pointee to store it in some self-referential data
    /// structure.
    ///
    /// # Examples
    /// ```rust
    /// use core::mem::MaybeUninit;
    /// use simple_safe_init::*;
    /// #[derive(Debug)]
    /// struct Count {
    ///     count: usize,
    /// }
    ///
    /// fn init_count<G>(this: InitMe<'_, Count, G>) -> InitProof<(), G> {
    ///     // SAFETY: We write to uninitialized memory using a raw pointer that is valid
    ///     unsafe { addr_of_mut!((*this.as_mut_ptr()).count).write(42); }
    ///     // SAFETY: We initialized all fields before
    ///     unsafe { this.assume_init() }
    /// }
    ///
    /// let count = Box::new(MaybeUninit::uninit());
    /// let count = init!(init_count(count));
    /// println!("{count:?}");
    /// ```
    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.ptr
    }
}

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

    unsafe fn ___init(self) -> Self::Init {
        InitProof {
            value: (),
            _phantom: PhantomData,
        }
    }

    unsafe fn ___as_mut_ptr(&mut self, _proof: &impl FnOnce(&Self::Raw)) -> *mut Self::Raw {
        self.ptr
    }

    unsafe fn ___i_have_read_the_documetation_and_verified_that_everything_is_correct() {}
}

/// A pointer to an Uninitialized `T` with a pinning guarantee, so the data cannot be moved
/// after initialization, if it is `!Unpin`.
///
/// *Implementation Detail:*
///
/// The second type parameter `G` is a guard type value. It is used to ensure that this object
/// returns a unique `InitProof<(), G>` that cannot be used to vouch for any other initialization
/// except this one.
/// See [here](#guard-parameter) for an explanation on this parameter.
pub struct PinInitMe<'a, T: ?Sized, G> {
    ptr: *mut T,
    // We need the correct variance, so we only accept the exact type for `G`. `T` and `'a` should
    // behave like `&'a mut T`.
    _phantom: PhantomData<(&'a mut T, fn(G) -> G)>,
}

impl<'a, T: ?Sized, G> Pointer for PinInitMe<'a, T, G> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:p}", self.ptr)
    }
}

impl<'a, T: ?Sized, G> InitPointer<'a, T, G> for PinInitMe<'a, T, G> {
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
    /// **Warning:** This function circumvents the protection created by this library. Try to avoid
    /// using this function.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use core::{marker::PhantomData, mem::MaybeUninit};
    /// use simple_safe_init::*;
    /// #[derive(Debug)]
    /// struct Count {
    ///     count: usize,
    ///     _pin: PhantomPinned,
    /// }
    ///
    /// fn init_count<G>(this: PinInitMe<'_, Count, G>) -> InitProof<(), G> {
    ///     // SAFETY: We write to uninitialized memory using a raw pointer that is valid
    ///     unsafe { addr_of_mut!((*this.as_mut_ptr()).count).write(42); }
    ///     // SAFETY: We write to uninitialized memory using a raw pointer that is valid
    ///     unsafe { addr_of_mut!((*this.as_mut_ptr())._pin).write(PhantomPinned); }
    ///     // SAFETY: We initialized all fields before
    ///     unsafe { this.assume_init() }
    /// }
    ///
    /// let count = Box::pin(MaybeUninit::uninit());
    /// let count = init!(init_count(count));
    /// println!("{count:?}");
    /// ```
    pub unsafe fn assume_init(self) -> InitProof<(), G> {
        InitProof {
            value: (),
            _phantom: PhantomData,
        }
    }

    /// Initialized the contents with the given value.
    ///
    /// This overwrites the memory pointed to without dropping the old value.
    ///
    /// # Examples
    /// ```rust
    /// use core::{marker::PhantomData, mem::MaybeUninit};
    /// use simple_safe_init::*;
    ///
    /// pin_data! {
    ///     #[derive(Debug)]
    ///     struct Count {
    ///         #pin
    ///         inner: usize,
    ///         #pin
    ///         _pin: PhantomPinned,
    ///     }
    /// }
    ///
    /// let count = Box::pin(MaybeUninit::uninit());
    /// let count = init! { count => Count {
    ///     ._pin = PhantomPinned;
    ///     PinInitMe::write(.inner, 42);
    /// }};
    /// println!("{count:?}");
    /// ```
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

    /// Gets a raw pointer to the pointee.
    ///
    /// Initially (after creation of an [`PinInitMe`]) the memory will be uninitialized. Because
    /// [`PinInitMe`] does not track partial initialization, using this function requires great care.
    /// Here are some of the hazards one could encounter:
    /// - overwriting a partially initialized value by calling [`PinInitMe::write`] (this will
    /// overwrite without calling drop),
    /// - calling [`PinInitMe::assume_init`] before the value is fully initialized (this is UB)
    ///
    /// This function is specifically designed to be used for:
    /// - careful manual initialization where init! is not sufficient (please check if this is
    /// really necessary),
    /// - getting access to the address of the pointee to store it in some self-referential data
    /// structure.
    ///
    /// # Examples
    /// ```rust
    /// use core::{marker::PhantomPinned, mem::MaybeUninit};
    /// use simple_safe_init::*;
    /// #[derive(Debug)]
    /// struct Count {
    ///     count: usize,
    ///     _pin: PhantomPinned,
    /// }
    ///
    /// fn init_count<G>(this: PinInitMe<'_, Count, G>) -> InitProof<(), G> {
    ///     // SAFETY: We write to uninitialized memory using a raw pointer that is valid
    ///     unsafe { addr_of_mut!((*this.as_mut_ptr()).count).write(42); }
    ///     // SAFETY: We write to uninitialized memory using a raw pointer that is valid
    ///     unsafe { addr_of_mut!((*this.as_mut_ptr())._pin).write(PhantomPinned); }
    ///     // SAFETY: We initialized all fields before
    ///     unsafe { this.assume_init() }
    /// }
    ///
    /// let count = Box::pin(MaybeUninit::uninit());
    /// let count = init!(init_count(count));
    /// println!("{count:?}");
    /// ```
    pub fn as_mut_ptr(&mut self) -> *mut T {
        unsafe { self.___as_mut_ptr(&|_| {}) }
    }
}

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

    unsafe fn ___init(self) -> Self::Init {
        InitProof {
            value: (),
            _phantom: PhantomData,
        }
    }

    unsafe fn ___as_mut_ptr(&mut self, _proof: &impl FnOnce(&Self::Raw)) -> *mut Self::Raw {
        self.ptr
    }

    unsafe fn ___i_have_read_the_documetation_and_verified_that_everything_is_correct() {}
}

unsafe impl<'a, T: ?Sized, G> PinnedPlace for PinInitMe<'a, T, G> {}

/// Proof to show, that a value was indeed initialized.
///
/// # Generic Arguments
/// The first parameter `T` is a wrapped value that was the normal return value of the function.
///
/// The second parameter `G` is a guard type value that is set up and used by the macros to ensure
/// sound initialization.
/// See [here](#guard-parameter) for an explanation on this parameter.
pub struct InitProof<T, G> {
    value: T,
    // correct invariance, we only accept the exact type G
    _phantom: PhantomData<fn(G) -> G>,
}

impl<T, G> InitProof<T, G> {
    /// Unwrap the actual result contained within and validate that the correct guard type was
    /// used.
    ///
    /// Users of the library generally do not need to call this.
    #[doc(hidden)]
    pub fn unwrap(self, _guard: G) -> T {
        self.value
    }
}

impl<G> InitProof<(), G> {
    /// Return a value instead of `()`.
    ///
    /// # Examples
    /// Initialized the contents with the given value.
    ///
    /// This overwrites the memory pointed to without dropping the old value.
    ///
    /// # Examples
    /// ```rust
    /// use core::{marker::PhantomData, mem::MaybeUninit};
    /// use simple_safe_init::*;
    /// #[derive(Debug)]
    /// struct Count {
    ///     inner: usize,
    ///     _pin: PhantomPinned,
    /// }
    ///
    /// fn init_count<G>(this: PinInitMe<'_, Count, G>) -> InitProof<*mut T, G> {
    ///     let ptr = this.as_mut_ptr();
    ///     init! { this => Self {
    ///         ._pin = PhantomPinned;
    ///         .inner = 42;
    ///     }}
    /// }
    ///
    /// let count = Box::new(MaybeUninit::uninit());
    /// let count = init! { count => Count {
    ///     ._pin = PhantomPinned;
    ///     InitMe::write(.inner, 42);
    /// }};
    /// println!("{count:?}");
    /// ```
    pub fn ret<T>(self, value: T) -> InitProof<T, G> {
        InitProof {
            value,
            _phantom: PhantomData,
        }
    }
}

/// Workaround to avoid a clippy error lint.
///
/// This prevents clippy denying code (diverging sub-expression) using the [`init!`]
/// macro when it checks for correct field initialization.
///
/// This is not really useful for normal code, because it always panics.
/// # Panics
/// Always panics.
pub fn conjure<T>() -> T {
    panic!("this function is not designed to be called!")
}
