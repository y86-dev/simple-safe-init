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
use core::{marker::PhantomData, mem::MaybeUninit, pin::Pin, ptr};
#[cfg(feature = "std")]
use std::alloc::AllocError;

#[cfg(feature = "alloc")]
extern crate alloc;
#[cfg(all(feature = "alloc", not(feature = "std")))]
use alloc::{boxed::Box, rc::Rc, sync::Arc};
#[cfg(all(not(feature = "alloc"), feature = "std"))]
use std::{boxed::Box, rc::Rc, sync::Arc};

/// Initialize a type on the stack. It will be pinned:
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
/// stack_init!(let foo = pin_init!(Foo {
///     a,
///     b: Bar {
///         x: 64,
///     },
/// }));
/// let foo: Result<Pin<&mut Foo>, !> = foo;
/// ```
#[macro_export]
macro_rules! stack_init {
    (let $var:ident = $val:expr) => {
        let mut $var = $crate::StackInit::uninit();
        let val = $val;
        let mut $var = unsafe { $crate::StackInit::init(&mut $var, val) };
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
/// let initializer = pin_init!(Foo {
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
///     pub fn new() -> impl PinInit<Self, !> {
///         pin_init!(Self {
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
/// #     pub fn new() -> impl PinInit<Self, !> {
/// #         pin_init!(Self {
/// #             a: 42,
/// #             b: Bar {
/// #                 x: 64,
/// #             },
/// #         })
/// #     }
/// # }
/// let foo = Box::init(Foo::new());
/// ```
/// They can also easily embed it into their own `struct`s:
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
/// #     pub fn new() -> impl PinInit<Self, !> {
/// #         pin_init!(Self {
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
///     pub fn new(other: u32) -> impl PinInit<Self, !> {
///         pin_init!(Self {
///             foo1: Foo::new(),
///             foo2: Foo::new(),
///             other,
///         })
///     }
/// }
/// ```
#[macro_export]
macro_rules! pin_init {
    ($(&$this:ident in)? $t:ident $(<$($generics:ty),* $(,)?>)? {
        $($field:ident $(: $val:expr)?),*
        $(,)?
    }) => {{
        let init = move |place: *mut $t $(<$($generics),*>)?| -> ::core::result::Result<(), _> {
            $(let $this = unsafe { ::core::ptr::NonNull::new_unchecked(place) };)?
            $(
                $(let $field = $val;)?
                // call the initializer
                // SAFETY: place is valid, because we are inside of an initializer closure, we return
                //         when an error/panic occurs.
                unsafe { $crate::PinInit::__init_pinned($field, ::core::ptr::addr_of_mut!((*place).$field))? };
                // create the drop guard
                // SAFETY: we forget the guard later when initialization has succeeded.
                let $field = unsafe { $crate::DropGuard::new(::core::ptr::addr_of_mut!((*place).$field)) };
            )*
            #[allow(unreachable_code, clippy::diverging_sub_expression)]
            if false {
                let _: $t $(<$($generics),*>)? = $t {
                    $($field: ::core::todo!()),*
                };
            }
            $(
                ::core::mem::forget($field);
            )*
            Ok(())
        };
        let init: $crate::PinInitClosure<_, $t $(<$($generics),*>)?, _> = unsafe { $crate::PinInitClosure::from_closure(init) };
        init
    }}
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
/// # let _: Result<Box<Foo>, AllocInitErr<!>> = Box::init(initializer);
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
///     pub fn new() -> impl Init<Self, !> {
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
/// #     pub fn new() -> impl Init<Self, !> {
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
/// They can also easily embed it into their own `struct`s:
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
/// #     pub fn new() -> impl Init<Self, !> {
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
///     pub fn new(other: u32) -> impl Init<Self, !> {
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
    ($t:ident $(<$($generics:ty),* $(,)?>)? {
        $($field:ident $(: $val:expr)?),*
        $(,)?
    }) => {{
        let init = move |place: *mut $t $(<$($generics),*>)?| -> ::core::result::Result<(), _> {
            $(
                $(let $field = $val;)?
                // call the initializer
                // SAFETY: place is valid, because we are inside of an initializer closure, we return
                //         when an error/panic occurs.
                unsafe { $crate::Init::__init($field, ::core::ptr::addr_of_mut!((*place).$field))? };
                // create the drop guard
                // SAFETY: we forget the guard later when initialization has succeeded.
                let $field = unsafe { $crate::DropGuard::new(::core::ptr::addr_of_mut!((*place).$field)) };
            )*
            #[allow(unreachable_code, clippy::diverging_sub_expression)]
            if false {
                let _: $t $(<$($generics),*>)? = $t {
                    $($field: ::core::todo!()),*
                };
            }
            $(
                // forget each guard
                ::core::mem::forget($field);
            )*
            Ok(())
        };
        let init: $crate::InitClosure<_, $t $(<$($generics),*>)?, _> = unsafe { $crate::InitClosure::from_closure(init) };
        init
    }}
}

#[cfg(feature = "never_type")]
type Never = !;

#[cfg(not(feature = "never_type"))]
type Never = core::convert::Infallible;

mod sealed {
    use super::*;
    pub trait Sealed {}

    impl Sealed for Direct {}
    impl Sealed for Closure {}
}

/// Marking ways of initialization, there exist two:
/// - [`Direct`],
/// - [`Closure`].
///
/// This is necessary, because otherwise the implementations would overlap.
pub trait InitWay: sealed::Sealed {}

impl InitWay for Direct {}
impl InitWay for Closure {}

/// Direct value based initialization.
pub struct Direct;
/// Initialization via closure that initializes each field.
pub struct Closure;

/// An initializer for `T`.
///
/// # Safety
/// The [`PinInit::__init_pinned`] function
/// - returns `Ok(())` iff it initialized every field of place,
/// - returns `Err(err)` iff it encountered an error and then cleaned place, this means:
///     - place can be deallocated without UB ocurring,
///     - place does not need to be dropped,
///     - place is not partially initialized.
pub unsafe trait PinInit<T, E = Never, Way: InitWay = Closure>: Sized {
    /// Initializes `place`.
    ///
    /// # Safety
    /// `place` is a valid pointer to uninitialized memory.
    /// The caller does not touch `place` when `Err` is returned, they are only permitted to
    /// deallocate.
    /// The place will not move, i.e. it will be pinned.
    unsafe fn __init_pinned(self, place: *mut T) -> Result<(), E>;
}

/// An initializer for `T`.
///
/// # Safety
/// The [`Init::__init`] function
/// - returns `Ok(())` iff it initialized every field of place,
/// - returns `Err(err)` iff it encountered an error and then cleaned place, this means:
///     - place can be deallocated without UB ocurring,
///     - place does not need to be dropped,
///     - place is not partially initialized.
///
/// The `__init_pinned` function from the supertrait [`PinInit`] needs to exectute the exact same
/// code as `__init`.
///
/// Contrary to its supertype [`PinInit<T, E, Way>`] the caller is allowed to
/// move the pointee after initialization.
pub unsafe trait Init<T, E = Never, Way: InitWay = Closure>: PinInit<T, E, Way> {
    /// Initializes `place`.
    ///
    /// # Safety
    /// `place` is a valid pointer to uninitialized memory.
    /// The caller does not touch `place` when `Err` is returned, they are only permitted to
    /// deallocate.
    unsafe fn __init(self, place: *mut T) -> Result<(), E>;
}

unsafe impl<T> PinInit<T, Never, Direct> for T {
    unsafe fn __init_pinned(self, place: *mut T) -> Result<(), Never> {
        // SAFETY: pointer valid as per function contract
        unsafe { place.write(self) };
        Ok(())
    }
}

unsafe impl<T> Init<T, Never, Direct> for T {
    unsafe fn __init(self, place: *mut T) -> Result<(), Never> {
        // SAFETY: pointer valid as per function contract
        unsafe { place.write(self) };
        Ok(())
    }
}

type Invariant<T> = PhantomData<fn(T) -> T>;

/// A closure initializer.
pub struct InitClosure<F, T, E>(F, Invariant<(T, E)>);

impl<T, E, F> InitClosure<F, T, E>
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
    /// - place may move after initialization
    pub const unsafe fn from_closure(f: F) -> Self {
        Self(f, PhantomData)
    }
}

unsafe impl<T, F, E> PinInit<T, E, Closure> for InitClosure<F, T, E>
where
    F: FnOnce(*mut T) -> Result<(), E>,
{
    unsafe fn __init_pinned(self, place: *mut T) -> Result<(), E> {
        (self.0)(place)
    }
}

unsafe impl<T, F, E> Init<T, E, Closure> for InitClosure<F, T, E>
where
    F: FnOnce(*mut T) -> Result<(), E>,
{
    unsafe fn __init(self, place: *mut T) -> Result<(), E> {
        (self.0)(place)
    }
}

/// A closure initializer for pinned data.
pub struct PinInitClosure<F, T, E>(F, Invariant<(T, E)>);

impl<T, E, F> PinInitClosure<F, T, E>
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
    pub const unsafe fn from_closure(f: F) -> Self {
        Self(f, PhantomData)
    }
}

unsafe impl<T, F, E> PinInit<T, E> for PinInitClosure<F, T, E>
where
    F: FnOnce(*mut T) -> Result<(), E>,
{
    unsafe fn __init_pinned(self, place: *mut T) -> Result<(), E> {
        (self.0)(place)
    }
}

/// When a value of this type is dropped, it drops something else.
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

/// Stack initializer helper type. See [`stack_init`].
pub struct StackInit<T>(MaybeUninit<T>, bool);

impl<T> Drop for StackInit<T> {
    fn drop(&mut self) {
        if self.1 {
            unsafe { self.0.assume_init_drop() };
        }
    }
}
impl<T> StackInit<T> {
    pub fn uninit() -> Self {
        Self(MaybeUninit::uninit(), false)
    }

    /// # Safety
    /// The caller ensures that `self` is on the stack and not accesible to **any** other code.
    pub unsafe fn init<E, Way: InitWay>(
        &mut self,
        init: impl PinInit<T, E, Way>,
    ) -> Result<Pin<&mut T>, E> {
        unsafe { init.__init_pinned(self.0.as_mut_ptr()) }?;
        self.1 = true;
        Ok(unsafe { Pin::new_unchecked(self.0.assume_init_mut()) })
    }
}

pub trait InPlaceInit<T>: Sized {
    type Error<E>;

    fn pin_init<E, Way: InitWay>(
        init: impl PinInit<T, E, Way>,
    ) -> Result<Pin<Self>, Self::Error<E>>;

    fn init<E, Way: InitWay>(init: impl Init<T, E, Way>) -> Result<Self, Self::Error<E>>;
}

/// Either an allocation error, or an initialization error.
#[derive(Debug)]
pub enum AllocInitErr<E> {
    Init(E),
    Alloc,
}

impl<E> From<Never> for AllocInitErr<E> {
    fn from(e: Never) -> Self {
        match e {}
    }
}

#[cfg(any(feature = "alloc", feature = "std"))]
impl<E> From<AllocError> for AllocInitErr<E> {
    fn from(_: AllocError) -> Self {
        Self::Alloc
    }
}

#[cfg(any(feature = "alloc", feature = "std"))]
impl<T> InPlaceInit<T> for Box<T> {
    type Error<E> = AllocInitErr<E>;

    fn pin_init<E, Way: InitWay>(
        init: impl PinInit<T, E, Way>,
    ) -> Result<Pin<Self>, Self::Error<E>> {
        let mut this = Box::try_new_uninit()?;
        let place = this.as_mut_ptr();
        // SAFETY: when init errors/panics, place will get deallocated but not dropped,
        // place is valid and will not be moved because of the into_pin
        unsafe { init.__init_pinned(place).map_err(AllocInitErr::Init)? };
        // SAFETY: all fields have been initialized
        Ok(Box::into_pin(unsafe { this.assume_init() }))
    }

    fn init<E, Way: InitWay>(init: impl Init<T, E, Way>) -> Result<Self, Self::Error<E>> {
        let mut this = Box::try_new_uninit()?;
        let place = this.as_mut_ptr();
        // SAFETY: when init errors/panics, place will get deallocated but not dropped,
        // place is valid
        unsafe { init.__init(place).map_err(AllocInitErr::Init)? };
        // SAFETY: all fields have been initialized
        Ok(unsafe { this.assume_init() })
    }
}

#[cfg(any(feature = "alloc", feature = "std"))]
impl<T> InPlaceInit<T> for Arc<T> {
    type Error<E> = AllocInitErr<E>;

    fn pin_init<E, Way: InitWay>(
        init: impl PinInit<T, E, Way>,
    ) -> Result<Pin<Self>, Self::Error<E>> {
        let mut this = Arc::try_new_uninit()?;
        let place = unsafe { Arc::get_mut_unchecked(&mut this) }.as_mut_ptr();
        // SAFETY: when init errors/panics, place will get deallocated but not dropped,
        // place is valid and will not be moved because of the into_pin
        unsafe { init.__init_pinned(place).map_err(AllocInitErr::Init)? };
        // SAFETY: all fields have been initialized
        Ok(unsafe { Pin::new_unchecked(this.assume_init()) })
    }

    fn init<E, Way: InitWay>(init: impl Init<T, E, Way>) -> Result<Self, Self::Error<E>> {
        let mut this = Arc::try_new_uninit()?;
        let place = unsafe { Arc::get_mut_unchecked(&mut this) }.as_mut_ptr();
        // SAFETY: when init errors/panics, place will get deallocated but not dropped,
        // place is valid
        unsafe { init.__init(place).map_err(AllocInitErr::Init)? };
        // SAFETY: all fields have been initialized
        Ok(unsafe { this.assume_init() })
    }
}

#[cfg(any(feature = "alloc", feature = "std"))]
impl<T> InPlaceInit<T> for Rc<T> {
    type Error<E> = AllocInitErr<E>;

    fn pin_init<E, Way: InitWay>(
        init: impl PinInit<T, E, Way>,
    ) -> Result<Pin<Self>, Self::Error<E>> {
        let mut this = Rc::try_new_uninit()?;
        let place = unsafe { Rc::get_mut_unchecked(&mut this) }.as_mut_ptr();
        // SAFETY: when init errors/panics, place will get deallocated but not dropped,
        // place is valid and will not be moved because of the into_pin
        unsafe { init.__init_pinned(place).map_err(AllocInitErr::Init)? };
        // SAFETY: all fields have been initialized
        Ok(unsafe { Pin::new_unchecked(this.assume_init()) })
    }

    fn init<E, Way: InitWay>(init: impl Init<T, E, Way>) -> Result<Self, Self::Error<E>> {
        let mut this = Rc::try_new_uninit()?;
        let place = unsafe { Rc::get_mut_unchecked(&mut this) }.as_mut_ptr();
        // SAFETY: when init errors/panics, place will get deallocated but not dropped,
        // place is valid
        unsafe { init.__init(place).map_err(AllocInitErr::Init)? };
        // SAFETY: all fields have been initialized
        Ok(unsafe { this.assume_init() })
    }
}

/// Marker trait for types that can be initialized by writing just zeroes.
///
/// # Safety
/// The bit pattern consisting of only zeroes must be a valid bit pattern for the type.
pub unsafe trait Zeroable {}

macro_rules! impl_zeroable {
    ($($t:ty),*) => {
        $(unsafe impl Zeroable for $t {})*
    };
}
impl_zeroable!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, Never);

unsafe impl<const N: usize, T: Zeroable> Zeroable for [T; N] {}

/// Create a new zeroed T
pub fn zeroed<T: Zeroable>() -> impl Init<T, Never> {
    unsafe {
        InitClosure::from_closure(|place: *mut T| {
            place.write_bytes(0, 1);
            Ok(())
        })
    }
}
