//! Module marking memory locations and pointer usable for (pinned) initialization.
//!
//! Normally it should not be necessary to use types of this module directly. If you need to
//! implement support for a smart pointer however, you need to implement [`PartialInitPlace`] for
//! it. Please read the hidden documentation in the code for each of the functions you need to
//! implement.

use super::{Guard, InitMe, InitPointer, PinInitMe};
#[cfg(feature = "alloc")]
use alloc::boxed::Box;
use core::{cell::UnsafeCell, mem::MaybeUninit, pin::Pin};

macro_rules! cfg_std {
    ($($stuff:item)*) => {
        $(
            // TODO change to docsrs
            #[cfg_attr(feature = "docsrs", doc(cfg(feature = "alloc")))]
            #[cfg(feature = "alloc")]
            $stuff
        )*
    }
}

/// Central trait to facilitate initialization. Every partially init-able place should implement this type.
///
/// If you need to implement this type, pay close attention to the comments on the methods of this
/// trait. You need to strictly adhere to the invariants listed, otherwise this library cannot
/// guarantee the soundness of your program.
///
/// # Safety
///
/// This trait requires that partially initialized values of type [`Self::Raw`] can be stored in
/// `Self` (essentially in [`MaybeUninit<T>`]). And initialized values of type [`Self::Raw`] can be
/// stored by [`Self::Init`].
///
/// It only makes sense to implement this type for
/// - direct memory locations (such as [`MaybeUninit<T>`]),
/// - smart pointers with write access to the pointee.
///
/// When implementing this type you need to be careful, as a faulty implementation can lead to UB
/// at a later point in time.
///
/// # Implementing
///
/// Care is needed, when implementing this trait for a smart pointer that allows multiple actors to access the
/// pointee at the same time. You will need to create a new type that only allows a single actor to
/// access the memory, use that to initialize it and then convert it to the actual smart pointer.
/// For this use-case, view the source code of [`UniqueArc<T>`] and [`UniqueRc<T>`].
///
/// [`UniqueArc<T>`]: super::unique::UniqueArc
/// [`UniqueRc<T>`]: super::unique::UniqueRc
pub unsafe trait PartialInitPlace {
    /// This is the type `Self` will become, when everything is fully initialized.
    type Init;
    /// This is the actual raw type that needs to be initialized. For smart pointers this is the
    /// pointee type.
    type Raw: ?Sized;
    /// This type should either be `PinInit<'a, Self::Raw, G>` or `InitMe<'a, Self::Raw, G>`.
    /// It is used to completely delegate the initialization to a single function.
    type InitMe<'a, G: Guard>: InitPointer<'a, Self::Raw, G>
    where
        Self: 'a;

    #[doc = include_str!("macro_only.md")]
    /// - `guard` is not accessible by unauthorized code.
    ///
    /// # Implementing
    ///
    /// This function should not be implemented manually!
    unsafe fn ___init_me<G: Guard>(this: &mut Self, guard: G) -> Self::InitMe<'_, G> {
        // SAFETY: macro only function
        unsafe { InitPointer::___new(Self::___as_mut_ptr(this, &|_| {}), guard) }
    }

    #[doc = include_str!("macro_only.md")]
    /// - all fields have been fully initialized.
    ///
    /// # Implementing
    ///
    /// When this function is being called, the callee can assume that all fields of the `Raw`
    /// pointee have been initialized via access through the raw pointer.
    ///
    /// Some smart pointers have layouts that depend upon the type parameters, take care of the
    /// translation in this function. For example: [`Box<T>`] and [`Box<MaybeUninit<T>>`] are
    /// **not** layout compatible, even though [`MaybeUninit<T>`] and `T` are! For more information
    /// on this, view [this UCG issue](https://github.com/rust-lang/unsafe-code-guidelines/issues/329) (unsafe code guidelines).
    ///
    /// If `Self` is a pointer type this function is not allowed to change the memory location
    /// of the initialized memory.
    ///
    /// No side effects allowed.
    ///
    /// [`Box<T>`]: [`alloc::boxed::Box<T>`]
    /// [`Box<MaybeUninit<T>>`]: [`core::mem::MaybeUninit<T>`]
    /// [`MaybeUninit<T>`]: [`core::mem::MaybeUninit<T>`]
    unsafe fn ___assume_init(this: Self) -> Self::Init;

    #[doc = include_str!("macro_only.md")]
    /// - the pointee is not moved out of the pointer.
    ///
    /// # Implementing
    ///
    /// - always return the same pointer,
    /// - no side effects allowed.
    unsafe fn ___as_mut_ptr(this: &mut Self, _proof: &impl FnOnce(&Self::Raw)) -> *mut Self::Raw;
}

/// Marker trait used to mark Places where the value cannot be moved out of. Example:
/// `Pin<Box<$value>>`.
///
/// # Safety
/// The value at this place cannot be moved.
pub unsafe trait PinnedPlace: PartialInitPlace {}

unsafe impl PartialInitPlace for ! {
    type Init = !;
    type Raw = !;
    type InitMe<'a, G: Guard> = InitMe<'a, !, G> where Self: 'a;

    unsafe fn ___assume_init(this: Self) -> Self::Init {
        this
    }

    unsafe fn ___as_mut_ptr(this: &mut Self, _proof: &impl FnOnce(&Self::Raw)) -> *mut Self::Raw {
        *this
    }
}

unsafe impl<T> PartialInitPlace for MaybeUninit<T> {
    type Init = T;
    type Raw = T;
    type InitMe<'a, G: Guard>
    = InitMe<'a, T, G>
    where
        Self: 'a
    ;

    unsafe fn ___assume_init(this: Self) -> Self::Init {
        // SAFETY: `T` has been initialized.
        unsafe { this.assume_init() }
    }

    unsafe fn ___as_mut_ptr(this: &mut Self, _proof: &impl FnOnce(&Self::Raw)) -> *mut Self::Raw {
        this.as_mut_ptr()
    }
}

cfg_std! {
    unsafe impl<T> PartialInitPlace for Box<MaybeUninit<T>> {
        type Init = Box<T>;
        type Raw = T;
        type InitMe<'a, G: Guard>
        = InitMe<'a, T, G>
        where
            Self: 'a
        ;

        unsafe fn ___assume_init(this: Self) -> Self::Init {
            // SAFETY: `T` has been initialized
            unsafe { this.assume_init() }
        }

        unsafe fn ___as_mut_ptr(this: &mut Self, _proof: &impl FnOnce(&Self::Raw)) -> *mut Self::Raw {
            MaybeUninit::as_mut_ptr(&mut **this)
        }
    }
}

unsafe impl<P, T> PartialInitPlace for Pin<P>
where
    P: PartialInitPlace + core::ops::DerefMut<Target = T>,
    P::Init: core::ops::Deref<Target = P::Raw>,
    T: PartialInitPlace<Raw = P::Raw>,
{
    type Init = Pin<P::Init>;
    type Raw = P::Raw;
    type InitMe<'a, G: Guard>
    = PinInitMe<'a, P::Raw, G>
    where
        Self: 'a
    ;

    unsafe fn ___assume_init(this: Self) -> Self::Init {
        // SAFETY: P::___assume_init will not change the address of the pointer, so we can re-pin the
        // returned smart pointer (it is a pointer, because it implements Deref)
        unsafe { Pin::new_unchecked(P::___assume_init(Pin::into_inner_unchecked(this))) }
    }

    unsafe fn ___as_mut_ptr(this: &mut Self, _proof: &impl FnOnce(&Self::Raw)) -> *mut Self::Raw {
        // SAFETY: macro never moves out of the pointer returned
        unsafe { T::___as_mut_ptr(Pin::get_unchecked_mut(this.as_mut()), _proof) }
    }
}

unsafe impl<P, T> PinnedPlace for Pin<P>
where
    P: PartialInitPlace + core::ops::DerefMut<Target = T>,
    P::Init: core::ops::Deref<Target = P::Raw>,
    T: PartialInitPlace<Raw = P::Raw>,
{
}

/// Helper trait used to allocate places.
///
/// Types marked with this trait can be allocated and initialized in one go using [`init!`].
///
/// [`init!`]: crate::init!
pub trait AllocablePlace {
    /// Error type that may occur when trying to allocate this type of place.
    type Error;
    /// The type of the alloced place
    type Alloced: PartialInitPlace + Sized;
    /// The type [`<Self::Alloced as PartialInitPlace>::Init`] gets converted to after
    /// initialization.
    ///
    /// [`<Self::Alloced as PartialInitPlace>::Init`]: PartialInitPlace::Init
    type Final;

    /// Allocate a place of this kind.
    ///
    /// # Errors
    ///
    /// This might fail when not enough memory of the specified kind is available.
    /// If it cannot fail, `Self::Error` should be `!` (the [never type](https://doc.rust-lang.org/reference/types/never.html)).
    fn allocate() -> Result<Self::Alloced, Self::Error>;

    /// After initialization operation.
    fn after_init(alloced: <Self::Alloced as PartialInitPlace>::Init) -> Self::Final;
}

impl<A: AllocablePlace> AllocablePlace for Pin<A>
where
    Pin<A::Alloced>: From<A::Alloced> + PartialInitPlace,
{
    type Error = A::Error;
    type Alloced = Pin<A::Alloced>;
    type Final = <Self::Alloced as PartialInitPlace>::Init;

    fn allocate() -> Result<Self::Alloced, A::Error> {
        Ok(Pin::from(A::allocate()?))
    }

    fn after_init(alloced: <Self::Alloced as PartialInitPlace>::Init) -> Self::Final {
        alloced
    }
}

cfg_std! {
    impl<T> AllocablePlace for Box<T> {
        type Error = alloc::alloc::AllocError;
        type Alloced = Box<MaybeUninit<T>>;
        type Final = Box<T>;

        fn allocate() -> Result<Self::Alloced, Self::Error> {
            Box::try_new_uninit()
        }

        fn after_init(alloced: <Self::Alloced as PartialInitPlace>::Init) -> Self::Final {
            alloced
        }
    }
}

/// # ⛔⛔⛔ **MACRO ONLY STRUCT** ⛔⛔⛔
///
/// This struct is only designed to be used by the macros of this library.
/// Using it directly might run into **unexpected and undefined behavior!**
///
/// I repeat: **DO NOT DECLARE/ALLOCATE/INITIALIZE THIS STRUCT MANUALLY!!**,
/// use the [`static_init!`] macro for that.
///
/// # Safety
///
/// DO NOT USE MANUALLY, use the [`static_init!`] macro instead.
///
/// [`static_init!`]: crate::static_init!
pub struct ___StaticInit<T> {
    inner: UnsafeCell<MaybeUninit<T>>,
}

impl<T> ___StaticInit<T> {
    #[doc = include_str!("macro_only.md")]
    /// - the returned value is initialized before it is used.
    pub const unsafe fn ___new() -> Self {
        Self {
            inner: UnsafeCell::new(MaybeUninit::uninit()),
        }
    }
    #[doc = include_str!("macro_only.md")]
    /// - pointer points to uninitialized memory.
    pub unsafe fn ___as_mut_ptr(&self) -> *mut T {
        unsafe {
            // SAFETY: the pointer is valid and not misused, as this is a macro
            // only function
            (*self.inner.get()).as_mut_ptr()
        }
    }
}

impl<T> core::ops::Deref for ___StaticInit<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe {
            // SAFETY: our type invariants dictates that our value has been initialized.
            (*self.inner.get()).assume_init_ref()
        }
    }
}

impl<T> core::ops::DerefMut for ___StaticInit<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            // SAFETY: our type invariant dictates that our value has been initialized.
            self.inner.get_mut().assume_init_mut()
        }
    }
}

/// # ⛔⛔⛔ **MACRO ONLY STRUCT** ⛔⛔⛔
///
/// This struct is only designed to be used by the macros of this library.
/// Using it directly might run into **unexpected and undefined behavior!**
///
/// I repeat: **DO NOT DECLARE/ALLOCATE/INITIALIZE THIS STRUCT MANUALLY!!**,
///
/// # Safety
///
/// DO NOT USE MANUALLY!
pub struct ___StackInit<T> {
    inner: MaybeUninit<T>,
    drop: bool,
}

impl<T> ___StackInit<T> {
    #[doc = include_str!("macro_only.md")]
    /// - the returned value is initialized before it is used.
    pub const unsafe fn ___new() -> Self {
        Self {
            inner: MaybeUninit::uninit(),
            drop: false,
        }
    }
    #[doc = include_str!("macro_only.md")]
    /// - pointer points to uninitialized memory.
    pub unsafe fn ___as_mut_ptr(&mut self) -> *mut T {
        self.inner.as_mut_ptr()
    }

    #[doc = include_str!("macro_only.md")]
    /// - `inner` has been initialized.
    pub unsafe fn ___assume_init_mut(&mut self) -> &mut T {
        self.drop = true;
        unsafe {
            // SAFETY: caller guarantees this is safe
            self.inner.assume_init_mut()
        }
    }
}

impl<T> Drop for ___StackInit<T> {
    fn drop(&mut self) {
        if self.drop {
            unsafe {
                // SAFETY: we have to be initialized for drop to be true
                self.inner.assume_init_drop();
            }
        }
    }
}

/// # ⛔⛔⛔ **MACRO ONLY TRAIT** ⛔⛔⛔
///
/// This trait is only designed to be implemented by the macros of this library.
/// Using it directly might run into **unexpected and undefined behavior!**
///
/// I repeat: **DO NOT IMPLEMENT THIS TRAIT MANUALLY!!**, use the [`pin_data!`] macro for that.
///
/// # Safety
///
/// DO NOT IMPLEMENT MANUALLY, use the [`pin_data!`] macro instead.
///
/// [`pin_data!`]: crate::pin_data!
pub unsafe trait ___PinData {
    /// # ⛔⛔⛔ **MACRO ONLY TYPE** ⛔⛔⛔
    ///
    /// This type is only designed to be implemented by the macros of this library.
    /// Using it directly might run into **unexpected and undefined behavior!**
    ///
    /// I repeat: **DO NOT USE THIS TYPE MANUALLY!!**
    type ___PinData;
}
