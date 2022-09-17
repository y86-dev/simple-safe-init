#![cfg_attr(not(feature = "std"), no_std)]
//
#![cfg_attr(feature = "never_type", feature(never_type))]
#![feature(allocator_api)]
#![cfg_attr(
    any(feature = "alloc", feature = "std"),
    feature(new_uninit, get_mut_unchecked)
)]
#![doc = include_str!("../README.md")]

#[cfg(feature = "alloc")]
use alloc::alloc::AllocError;
use core::{
    fmt,
    marker::PhantomData,
    mem::MaybeUninit,
    ops::{Deref, DerefMut},
    pin::Pin,
    ptr::{self, NonNull},
};
#[cfg(feature = "std")]
use std::alloc::AllocError;

#[cfg(feature = "alloc")]
extern crate alloc;
#[cfg(all(feature = "alloc", not(feature = "std")))]
use alloc::{boxed::Box, rc::Rc, sync::Arc};
#[cfg(all(not(feature = "alloc"), feature = "std"))]
use std::{boxed::Box, rc::Rc, sync::Arc};

#[macro_export]
macro_rules! stack_init {
    (let $var:ident $(: $t:ty;)? <- $val:expr) => {
        // SAFETY: `__store` is not accessible
        let mut __store = unsafe { $crate::StackInit$(::<$t>)?::uninit() };
        // SAFETY: `__store` is not accessible
        let $var = unsafe { $crate::StackInit::init(&mut __store, $val)? };
    };
    (let _ $(: $t:ty;)? <- $val:expr) => {
        // SAFETY: `__store` is not accessible
        let mut __store = unsafe { $crate::StackInit$(::<$t>)?::uninit() };
        // SAFETY: `__store` is not accessible
        let _ = unsafe { $crate::StackInit::init(&mut __store, $val)? };
    };
    (let $var:ident $(: $t:ty)? = $val:expr) => {
        let mut __store = $val;
        // SAFETY: `__store` is not accessible
        let $var = unsafe { ::core::pin::Pin::new_unchecked(&mut __store) };
    };
    (let _ $(: $t:ty)? = $val:expr) => {
        let mut __store = $val;
        // SAFETY: `__store` is not accessible
        let _ = unsafe { ::core::pin::Pin::new_unchecked(&mut __store) };
    };
}

/// Construct an in-place initializer for structs.
///
/// The syntax is identical to a normal struct initializer:
/// ```rust
/// # #![feature(never_type)]
/// # use simple_safe_init::*;
/// # use core::pin::Pin;
/// struct Foo {
///     a: usize,
///     b: Bar,
/// }
///
/// struct Bar {
///     x: u32,
/// }
///
/// let a = 42;
///
/// let initializer = init!(Foo {
///     a,
///     b: Bar {
///         x: 64,
///     },
/// });
/// # let _: Result<Pin<Box<Foo>>, AllocInitErr<!>> = Box::pin_init(initializer);
/// ```
/// Arbitrary rust expressions can be used to set the value of a variable.
///
/// # Init-functions
///
/// When working with this library it is often desired to let others construct your types without
/// giving access to all fields. This is where you would normally write a plain function `new`
/// that would return a new instance of your type. With this library that is also possible, however
/// there are a few extra things to keep in mind.
///
/// To create an initializer function, simple declare it like this:
/// ```rust
/// # #![feature(never_type)]
/// # use simple_safe_init::*;
/// # use core::pin::Pin;
/// # struct Foo {
/// #     a: usize,
/// #     b: Bar,
/// # }
/// # struct Bar {
/// #     x: u32,
/// # }
///
/// impl Foo {
///     pub fn new() -> impl Initializer<Self, !> {
///         init!(Self {
///             a: 42,
///             b: Bar {
///                 x: 64,
///             },
///         })
///     }
/// }
/// ```
/// Users of `Foo` can now create it like this:
/// ```rust
/// # #![feature(never_type)]
/// # use simple_safe_init::*;
/// # use core::pin::Pin;
/// # struct Foo {
/// #     a: usize,
/// #     b: Bar,
/// # }
/// # struct Bar {
/// #     x: u32,
/// # }
/// # impl Foo {
/// #     pub fn new() -> impl Initializer<Self, !> {
/// #         init!(Self {
/// #             a: 42,
/// #             b: Bar {
/// #                 x: 64,
/// #             },
/// #         })
/// #     }
/// # }
/// let foo = Box::init(Foo::new());
/// ```
/// They can also easily embedd it into their `struct`s:
/// ```rust
/// # #![feature(never_type)]
/// # use simple_safe_init::*;
/// # use core::pin::Pin;
/// # struct Foo {
/// #     a: usize,
/// #     b: Bar,
/// # }
/// # struct Bar {
/// #     x: u32,
/// # }
/// # impl Foo {
/// #     pub fn new() -> impl Initializer<Self, !> {
/// #         init!(Self {
/// #             a: 42,
/// #             b: Bar {
/// #                 x: 64,
/// #             },
/// #         })
/// #     }
/// # }
/// struct FooContainer {
///     foo1: Foo,
///     foo2: Foo,
///     other: u32,
/// }
///
/// impl FooContainer {
///     pub fn new(other: u32) -> impl Initializer<Self, !> {
///         init!(Self {
///             foo1: Foo::new(),
///             foo2: Foo::new(),
///             other,
///         })
///     }
/// }
/// ```
#[macro_export]
macro_rules! init {
    ($($this:ident)? <- $t:ident $(<$($generics:ty),* $(,)?>)? {
        $($inner:tt)*
    }) => {{
        let init = move |place: *mut $t $(<$($generics),*>)?| -> ::core::result::Result<(), _> {
            $(let $this = unsafe { $crate::InitPtr::new_unchecked(place) };)?
            $crate::init!(@place(place) @typ($t $(<$($generics),*>)?) @parse($($inner)*) @check() @forget());
        };
        let init = unsafe { $crate::Init::from_closure(init) };
        init
    }};
    (@place($place:ident) @typ($t:ident $(<$($generics:ty),*>)?) @parse() @check($($check:tt)*) @forget($($forget:tt)*)) => {
        #[allow(unreachable_code, clippy::diverging_sub_expression)]
        if false {
            let _: $t $(<$($generics),*>)? = $t {
                $($check)*
            };
        }
        $($forget)*
        return Ok(());
    };
    (@place($place:ident) @typ($t:ident $(<$($generics:ty),*>)?) @parse($field:ident <- $val:expr$(, $($tail:tt)*)?) @check($($check:tt)*) @forget($($forget:tt)*)) => {
        let $field = $val;
        // SAFETY: place is valid, because we are inside of an initializer closure, we return
        //         when an error/panic occurs.
        unsafe { $crate::Initializer::__init($field, ::core::ptr::addr_of_mut!((*$place).$field))? };
        // create the drop guard
        // SAFETY: we forget the guard later when initialization has succeeded.
        let $field = unsafe { $crate::DropGuard::new(::core::ptr::addr_of_mut!((*$place).$field)) };
        $crate::init!(@place($place) @typ($t $(<$($generics),*>)?) @parse($($($tail)*)?) @check($field: ::core::todo!(), $($check)*) @forget(::core::mem::forget($field); $($forget)*));
    };
    (@place($place:ident) @typ($t:ident $(<$($generics:ty),*>)?) @parse($field:ident $(: $val:expr)?$(, $($tail:tt)*)?) @check($($check:tt)*) @forget($($forget:tt)*)) => {
        $(let $field = $val;)?
        // write the value directly
        unsafe { ::core::ptr::addr_of_mut!((*$place).$field).write($field) };
        // create the drop guard
        // SAFETY: we forget the guard later when initialization has succeeded.
        let $field = unsafe { $crate::DropGuard::new(::core::ptr::addr_of_mut!((*$place).$field)) };
        $crate::init!(@place($place) @typ($t $(<$($generics),*>)?) @parse($($($tail)*)?) @check($field: ::core::todo!(), $($check)*) @forget(::core::mem::forget($field); $($forget)*));
    };
}

/// An initializer for `T`.
///
/// # Safety
/// The [`Initializer::__init`] function
/// - returns `Ok(())` iff it initialized every field of place,
/// - returns `Err(err)` iff it encountered an error and then cleaned place, this means:
///     - place can be deallocated without UB ocurring,
///     - place does not need to be dropped,
///     - place is not partially initialized.
pub unsafe trait Initializer<T, E> {
    /// Initializes `place`.
    ///
    /// # Safety
    /// `place` is a valid pointer to uninitialized memory.
    /// - The caller does not touch `place` when `Err` is returned, they are only permitted to deallocate.
    /// - If `T: !Unpin` then `place` will need to be pinned after returning `Ok(())`
    unsafe fn __init(self, place: *mut T) -> Result<(), E>;
}

type Invariant<T> = PhantomData<fn(T) -> T>;

/// A closure initializer.
pub struct Init<F, T, E>(F, Invariant<(T, E)>);

impl<T, E, F> Init<F, T, E>
where
    F: FnOnce(*mut T) -> Result<(), E>,
{
    /// Creates a new Init from the given closure
    ///
    /// # Safety
    /// The closure
    /// - returns `Ok(())` iff it initialized every field of place,
    /// - returns `Err(err)` iff it encountered an error and then cleaned place, this means:
    ///     - place can be deallocated without UB ocurring,
    ///     - place does not need to be dropped,
    ///     - place is not partially initialized.
    /// - place will not move after initialization if `T: !Unpin`
    pub const unsafe fn from_closure(f: F) -> Self {
        Self(f, PhantomData)
    }
}

unsafe impl<T, F, E> Initializer<T, E> for Init<F, T, E>
where
    F: FnOnce(*mut T) -> Result<(), E>,
{
    unsafe fn __init(self, place: *mut T) -> Result<(), E> {
        (self.0)(place)
    }
}

pub struct InitPtr<T: ?Sized>(NonNull<T>);

impl<T: ?Sized> fmt::Debug for InitPtr<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Pointer::fmt(&self.as_ptr(), f)
    }
}

impl<T: ?Sized> fmt::Pointer for InitPtr<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Pointer::fmt(&self.as_ptr(), f)
    }
}

impl<T: ?Sized> Clone for InitPtr<T> {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl<T: ?Sized> Copy for InitPtr<T> {}

impl<T: ?Sized> Deref for InitPtr<T> {
    type Target = NonNull<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<T: ?Sized> DerefMut for InitPtr<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: ?Sized> From<InitPtr<T>> for NonNull<T> {
    fn from(ptr: InitPtr<T>) -> Self {
        ptr.0
    }
}

impl<T: ?Sized> InitPtr<T> {
    pub unsafe fn new_unchecked(ptr: *mut T) -> Self {
        Self(NonNull::new_unchecked(ptr))
    }
}

/// When a value of this type is dropped, it drops a `T`.
pub struct DropGuard<T: ?Sized>(*mut T);

impl<T: ?Sized> DropGuard<T> {
    /// Creates a new [`DropGuard<T>`]. It will [`ptr::drop_in_place`] `ptr` when it gets dropped.
    ///
    /// # Safety
    /// `ptr` must be a valid poiner.
    ///
    /// It is the callers responsibility that `self` will only get dropped if the pointee of `ptr`:
    /// - has not been dropped,
    /// - is not accesible by any other means,
    /// - will not be dropped by any other means.
    pub unsafe fn new(ptr: *mut T) -> Self {
        Self(ptr)
    }
}

impl<T: ?Sized> Drop for DropGuard<T> {
    fn drop(&mut self) {
        // SAFETY: safe as a `DropGuard` can only be constructed using the unsafe new function.
        unsafe { ptr::drop_in_place(self.0) }
    }
}

/// Stack allocated and initialized data
pub struct StackInit<T>(MaybeUninit<T>, bool);

impl<T> Drop for StackInit<T> {
    fn drop(&mut self) {
        if self.1 {
            unsafe { self.0.assume_init_drop() };
        }
    }
}

impl<T> StackInit<T> {
    pub unsafe fn uninit() -> Self {
        Self(MaybeUninit::uninit(), false)
    }

    pub unsafe fn init<E>(&mut self, init: impl Initializer<T, E>) -> Result<Pin<&mut T>, E> {
        unsafe { init.__init(self.0.as_mut_ptr())? };
        self.1 = true;
        Ok(unsafe { Pin::new_unchecked(self.0.assume_init_mut()) })
    }
}

pub unsafe trait InPlaceInit<T>: Sized + Deref<Target = T> {
    type Error<E>;

    fn pin_init<E>(init: impl Initializer<T, E>) -> Result<Pin<Self>, Self::Error<E>>;

    fn init<E>(init: impl Initializer<T, E>) -> Result<Self, Self::Error<E>>
    where
        T: Unpin,
    {
        InPlaceInit::pin_init(init).map(Pin::into_inner)
    }
}

#[derive(Debug)]
pub enum AllocInitErr<E> {
    Init(E),
    Alloc,
}

#[cfg(feature = "never_type")]
impl<E> From<!> for AllocInitErr<E> {
    fn from(e: !) -> Self {
        e
    }
}

#[cfg(any(feature = "alloc", feature = "std"))]
impl<E> From<AllocError> for AllocInitErr<E> {
    fn from(_: AllocError) -> Self {
        Self::Alloc
    }
}

macro_rules! impl_in_place_init {
    ($($t:ident[$get_mut:ident]),*) => {
        $(
            #[cfg(any(feature = "alloc", feature = "std"))]
            unsafe impl<T> InPlaceInit<T> for $t<T> {
                type Error<E> = AllocInitErr<E>;

                fn pin_init<E>(init: impl Initializer<T, E>) -> Result<Pin<Self>, Self::Error<E>> {
                    let mut this = $t::try_new_uninit()?;
                    #[allow(unused_unsafe)]
                    let place = unsafe { $t::$get_mut(&mut this) }.as_mut_ptr();
                    // SAFETY: when init errors/panics, place will get deallocated but not dropped,
                    // place is valid and will not be moved because of the Pin::new_unchecked
                    unsafe { init.__init(place).map_err(AllocInitErr::Init)? };
                    // SAFETY: all fields have been initialized
                    Ok(unsafe { Pin::new_unchecked(this.assume_init())})
                }
            }
        )*
    };
}

impl_in_place_init!(
    Box[deref_mut],
    Arc[get_mut_unchecked],
    Rc[get_mut_unchecked]
);
