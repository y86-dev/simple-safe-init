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

#![feature(new_uninit, generic_associated_types)]
#![deny(unsafe_op_in_unsafe_fn)]

pub mod init;
pub mod place;

pub use init::*;

mod tests;

/// # Overview
/// This macro is the core of this library, there are several ways to initialize fields of structs.
/// Here is an example:
/// ```rust,no_run
/// use simple_safe_init::*;
/// use core::mem::MaybeUninit;
///
/// pin_data! {
///     struct Foo<T> {
///         msg: String,
///         limit: usize,
///         value: T,
///         inner: InnerFoo,
///         bar: isize,
///     }
/// }
///
/// struct InnerFoo {
///     x: u8,
/// }
///
/// fn init_limit<G>(mut limit: InitMe<'_, usize, G>, limit_type: u8) -> InitProof<(), G> {
///     extern "C" {
///         fn __init_limit(ptr: *mut usize, typ: u8);
///     }
///     unsafe {
///         // SAFETY: `__init_limit` initializes the pointee
///         __init_limit(limit.as_mut_ptr(), limit_type);
///         limit.assume_init()
///     }
/// }
///
/// macro_rules! init_inner {
///     ($inner:ident, $val:expr) => {
///         // this macro needs to return an expression that returns an InitProof
///         $inner.write($val)
///     };
/// }
///
/// fn init_bar<G>(bar: InitMe<'_, isize, G>) -> InitProof<isize, G> {
///     bar.write(1).ret(1)
/// }
///
/// let foo = Box::pin(MaybeUninit::<Foo<f64>>::uninit());
/// // first specify the expression you want to initialize, then specify the exact type with generics
/// init! { foo => Foo<f64> {
///     // just normally assign the variable
///     .msg = "Hello World".to_owned();
///     // use a delegation function
///     init_limit(.limit, 0);
///     // use a delegation macro
///     init_inner!(.inner, InnerFoo { x: 16 });
///     // you can use already initialized values
///     .value = (*limit) as f64;
///     // get the return value from an init function
///     ~let val = init_bar(.bar);
///     // you can use normal macros (as long as they do not start with `.` and then an ident):
///     assert_eq!(val, 1);
///     // you can use arbitrary rust statements ...
/// }};
/// ```
#[macro_export]
macro_rules! init {
    ($var:expr => $struct:ident $(<$($generic:ty),*>)? { $($tail:tt)* }) => {
        match $var {
            mut var => {
                fn no_warn<___T>(_: &mut ___T) {}
                no_warn(&mut var);
                $crate::init!(@inner(var _is_pinned () $struct $(<$($generic),*>)?) $($tail)*);
                unsafe {
                    // SAFETY: The pointee of `var` has been fully initialized, if this part is
                    // reachable and no compile error exist.
                    $crate::place::PartialInitPlace::___init(var)
                }
            }
        }
    };
    ($func:ident $(:: $(<$($args:ty),*$(,)?>::)? $path:ident)*($var:expr $(, $($rest:tt)*)?)) => {
        match $var {
            mut var => {
                {
                    struct ___LocalGuard;
                    let value = unsafe {
                        // SAFETY: we own `var` and assume it is initialized below
                        $crate::place::PartialInitPlace::___init_me(&mut var, ___LocalGuard)
                    };
                    let guard = ___LocalGuard;
                    {
                        struct ___LocalGuard;
                        let () = $crate::init::InitProof::unwrap($func $(:: $(<$($args),*>::)? $path)* (value $(, $($rest)*)?), guard);
                    }
                }
                unsafe {
                    // SAFETY: The pointee was initialized by the function above and the InitProof
                    // was valid.
                    $crate::place::PartialInitPlace::___init(var)
                }
            }
        }
    };
    // when there is no input left, construct a struct initializer with all of the fields
    // mentioned. If one is missing or a duplicate, the compiler will complain.
    // We do this inside of a closure, because we do not want to really create this struct. Also,
    // the values of the fields are `todo!()` thus the allow unreachable.
    (@inner($var:ident $pin:ident ($($inner:tt)*) $name:ident $(<$($generic:ty),*>)?)) => {
        #[allow(unreachable_code)]
        let ____check_all_init = || {
            let _struct: $name $(<$($generic),*>)? = $name {
                $($inner)*
            };
        };
    };
    // a normal assignment, use raw pointers to set the value.
    (@inner($var:ident $pin:ident ($($inner:tt)*) $name:ident $(<$($generic:ty),*>)?)
        .$field:ident = $val:expr;
        $($tail:tt)*
    ) => {
        match $val {
            val => unsafe {
                // SAFETY: ___as_mut_ptr returns a valid pointer that points to possibly uninit
                // memory. we only use ptr::write, which is allowed
                ::core::ptr::write(
                    ::core::ptr::addr_of_mut!(
                        (*$crate::place::PartialInitPlace::___as_mut_ptr(&mut $var, &|_: &$name $(<$($generic),*>)?|  {})).$field
                    ),
                    val
                );
            }
        }
        let $field = {
            unsafe {
                // we initialized the memory above, so we can now create a reference
                &mut *::core::ptr::addr_of_mut!((*$crate::place::PartialInitPlace::___as_mut_ptr(&mut $var, &|_: &$name $(<$($generic),*>)?| {})).$field)
            }
        };
        #[allow(unused_variables)]
        let $field = $field;
        $crate::init!(@inner($var $pin ($($inner)* $field: ::core::todo!(),) $name $(<$($generic),*>)?) $($tail)*);
    };
    // a function call initializing a single field, we cannot use the `path` meta-variable type,
    // because `(` is not allowed after that :(
    (@inner($var:ident $pin:ident ($($inner:tt)*) $name:ident $(<$($generic:ty),*>)?)
        $(~let $binding:pat = )?$func:ident $(:: $(<$($args:ty),*$(,)?>::)? $path:ident)*(.$field:ident $($rest:tt)*);
        $($tail:tt)*
    ) => {
        $crate::init!(@init_call($var, $name $(<$($generic),*>)?, $field, field_place, ($func $(:: $(<$($args),*>::)? $path)*(field_place $($rest)*)), $($binding)?));
        $crate::init!(@inner($var $pin ($($inner)* $field: ::core::todo!(),) $name $(<$($generic),*>)?) $($tail)*);
    };
    // an unsafe function initializing a single field.
    (@inner($var:ident $pin:ident ($($inner:tt)*) $name:ident $(<$($generic:ty),*>)?)
        $(~let $binding:pat = )?unsafe { $func:ident $(:: $(<$($args:ty),*$(,)?>::)? $path:ident)*(.$field:ident $($rest:tt)*) };
        $($tail:tt)*
    ) => {
        $crate::init!(@init_call($var, $name $(<$($generic),*>)?, $field, field_place, (unsafe { $func $(:: $(<$($args),*>::)? $path)*(field_place $($rest)*) }), $($binding)?));
        $crate::init!(@inner($var $pin ($($inner)* $field: ::core::todo!(),) $name $(<$($generic),*>)?) $($tail)*);
    };
    // a macro call initializing a single field
    (@inner($var:ident $pin:ident ($($inner:tt)*) $name:ident $(<$($generic:ty),*>)?)
        $(~let $binding:pat = )?$func:ident $(:: $(<$($args:ty),*$(,)?>::)? $path:ident)*!(.$field:ident $($rest:tt)*);
        $($tail:tt)*
    ) => {
        $crate::init!(@init_call($var, $name $(<$($generic),*>)?, $field, field_place, ($func $(:: $(<$($args),*>::)? $path)*!(field_place $($rest)*)), $($binding)?));
        $crate::init!(@inner($var $pin ($($inner)* $field: ::core::todo!(),) $name $(<$($generic),*>)?) $($tail)*);
    };
    // an async function call initializing a single field
    (@inner($var:ident $pin:ident ($($inner:tt)*) $name:ident $(<$($generic:ty),*>)?)
        $(~let $binding:pat = )?$func:ident $(:: $(<$($args:ty),*$(,)?>::)? $path:ident)*(.$field:ident $($rest:tt)*).await;
        $($tail:tt)*
    ) => {
        $crate::init!(@init_call($var, $name $(<$($generic),*>)?, $field, field_place, ($func $(:: $(<$($args),*>::)? $path)*(field_place $($rest)*).await), $($binding)?));
        $crate::init!(@inner($var $pin ($($inner)* $field: ::core::todo!(),) $name $(<$($generic),*>)?) $($tail)*);
    };
    // a normal statement that will be executed as-is.
    (@inner($var:ident $pin:ident ($($inner:tt)*) $($name:tt)*)
        $st:stmt;
        $($tail:tt)*
    ) => {
        $st
        $crate::init!(@inner($var $pin ($($inner)*) $($name)*) $($tail)*);
    };
    // generalized function/macro call helper
    (@init_call($var:ident, $name:ident $(<$($generic:ty),*>)?, $field:ident, $field_place:ident, ($($call:tt)*), $($binding:pat)?)) => {
        let result;
        {
            // this type is used as the guard parameter on `(Pin)InitMe` and ensures that we
            // definitely initialize the specified field. we scope it here, to ensure no usage
            // outside of this macro.
            #[doc(hidden)]
            struct ___LocalGuard;
            let mut var = Some(&$var);
            var = None;
            // get the correct pin projection (handled by the ___PinData type)
            let $field_place = unsafe {
                <$name $(<$($generic),*>)? as $crate::place::___PinData>::___PinData::$field(
                    ::core::ptr::addr_of_mut!((*$crate::place::PartialInitPlace::___as_mut_ptr(&mut $var, &|_: &$name $(<$($generic),*>)?| {})).$field),
                    var,
                    ___LocalGuard,
                )
            };
            // create a guard that will be used later, as we want to shadow the type definition to
            // prevent misuse by a proc macro
            let guard = ___LocalGuard;
            {
                // shadow the type def
                #[doc(hidden)]
                struct ___LocalGuard;
                // unwrap the value produced by the function immediately, do not give access to the
                // raw InitProof. Validate using the guard, if guard would be used a second time,
                // then a move error would occur.
                result = $crate::init::InitProof::unwrap($($call)*, guard);
            }
        }
        $(let $binding = result;)?
        // create a mutable reference to the object, it can now be used, because it was initialized.
        let $field = {
            unsafe {
                &mut *::core::ptr::addr_of_mut!((*$crate::place::PartialInitPlace::___as_mut_ptr(&mut $var, &|_: &$name $(<$($generic),*>)?| {})).$field)
            }
        };
        // do not complain, if it is not used.
        #[allow(unused_variables)]
        let $field = $field;
    };
}

#[macro_export]
macro_rules! pin_data {
    (
        $(#[$struct_attr:meta])*
        $vis:vis struct $name:ident $(<$($($life:lifetime),+ $(,)?)? $($generic:ident $(: ?$sized:ident)?),* $(,)?>)? $(where $($whr:path : $bound:ty),* $(,)?)? {
            $(
                $(#$pin:ident)?
                $(#[$attr:meta])*
                $fvis:vis $field:ident : $type:ty
            ),*
            $(,)?
        }
    ) => {
        $(#[$struct_attr])*
        $vis struct $name $(<$($($life),+ ,)? $($generic $(: ?$sized)?),*>)? $(where $($whr : $bound),*)? {
            $(
                $(#[$attr])*
                $fvis $field: $type
            ),*
        }

        const _: () = {
            $vis struct ___ThePinData;

            impl ___ThePinData {
                $(
                    $crate::pin_data!(@make_fn(($fvis) $($pin)? $field: $type));
                )*
            }

            unsafe impl$(<$($($life),+ ,)? $($generic $(: ?$sized)?),*>)? $crate::place::___PinData for $name$(<$($($life),+ ,)? $($generic),*>)? {
                type ___PinData = ___ThePinData;
            }
        };
    };
    (@make_fn(($vis:vis) pin $field:ident : $type:ty)) => {
        $vis unsafe fn $field<'a, T, P: $crate::place::PinnedPlace, G>(ptr: *mut T, _place: Option<&P>, guard: G) -> $crate::init::PinInitMe<'a, T, G> {
            unsafe { $crate::init::PinInitMe::___new(ptr, guard) }
        }
    };
    (@make_fn(($vis:vis) $field:ident : $type:ty)) => {
        $vis unsafe fn $field<'a, T,P: $crate::place::PartialInitPlace, G>(ptr: *mut T, _place: Option<&P>, guard: G) -> $crate::init::InitMe<'a, T, G> {
            unsafe { $crate::init::InitMe::___new(ptr, guard) }
        }
    };
}
