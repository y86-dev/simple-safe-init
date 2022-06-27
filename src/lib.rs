#![feature(new_uninit, generic_associated_types)]
#![deny(unsafe_op_in_unsafe_fn)]
pub mod init;
pub mod place;

pub use init::*;

#[macro_export]
macro_rules! init {
    ($var:expr => $struct:ident $(<$($generic:ty),*>)? { $($tail:tt)* }) => {
        match $var {
            mut var => {
                fn no_warn<___T>(_: &mut ___T) {}
                no_warn(&mut var);
                $crate::init!(@inner(var _is_pinned () $struct $(<$($generic),*>)?) $($tail)*);
                unsafe {
                    $crate::place::___PlaceInit::___init(var)
                }
            }
        }
    };
    ($func:ident $(:: $(<$($args:ty),*$(,)?>::)? $path:ident)*($var:expr; $($rest:tt)*)) => {
        match $var {
            mut var => {
                let () = $crate::init::InitProof::unwrap($func $(:: $(<$($args),*>::)? $path)* (unsafe {
                    $crate::place::___PlaceInit::___init_me(&mut $var)
                } $($rest)*));
                unsafe {
                    $crate::place::___PlaceInit::___init($var)
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
                // SAFETY: we are
                // - not inspecting the pointee (might be uninit)
                // - not moving the value (might be pinned)
                // - not expanding any uncontrollable macro input
                ::core::ptr::write(
                    ::core::ptr::addr_of_mut!(
                        (*$crate::place::___PlaceInit::___as_mut_ptr(&mut $var, &|_: &$name $(<$($generic),*>)?|  {})).$field
                    ),
                    val
                );
            }
        }
        let $field = {
            unsafe {
                &mut *::core::ptr::addr_of_mut!((*$crate::place::___PlaceInit::___as_mut_ptr(&mut $var, &|_: &$name $(<$($generic),*>)?| {})).$field)
            }
        };
        #[allow(unused_variables)]
        let $field = $field;
        $crate::init!(@inner($var $pin ($($inner)* $field: ::core::todo!(),) $name $(<$($generic),*>)?) $($tail)*);
    };
    // a function call initializing a single field, we cannot use the `path` metavariable type,
    // because `(` is not allowed after that :(
    (@inner($var:ident $pin:ident ($($inner:tt)*) $name:ident $(<$($generic:ty),*>)?)
        $(let $binding:pat = )?$func:ident $(:: $(<$($args:ty),*$(,)?>::)? $path:ident)*(.$field:ident $($rest:tt)*);
        $($tail:tt)*
    ) => {
        $crate::init!(@init_call($var, $name $(<$($generic),*>)?, $field, field_place, ($func $(:: $(<$($args),*>::)? $path)*(field_place $($rest)*)), $($binding)?));
        $crate::init!(@inner($var $pin ($($inner)* $field: ::core::todo!(),) $name $(<$($generic),*>)?) $($tail)*);
    };
    // an unsafe function initializing a single field.
    (@inner($var:ident $pin:ident ($($inner:tt)*) $name:ident $(<$($generic:ty),*>)?)
        $(let $binding:pat = )?unsafe { $func:ident $(:: $(<$($args:ty),*$(,)?>::)? $path:ident)*(.$field:ident $($rest:tt)*) };
        $($tail:tt)*
    ) => {
        $crate::init!(@init_call($var, $name $(<$($generic),*>)?, $field, field_place, (unsafe { $func $(:: $(<$($args),*>::)? $path)*(field_place $($rest)*) }), $($binding)?));
        $crate::init!(@inner($var $pin ($($inner)* $field: ::core::todo!(),) $name $(<$($generic),*>)?) $($tail)*);
    };
    // a macro call initializing a single field
    (@inner($var:ident $pin:ident ($($inner:tt)*) $name:ident $(<$($generic:ty),*>)?)
        $(let $binding:pat = )?$func:ident $(:: $(<$($args:ty),*$(,)?>::)? $path:ident)*!(.$field:ident $($rest:tt)*);
        $($tail:tt)*
    ) => {
        $crate::init!(@init_call($var, $name $(<$($generic),*>)?, $field, field_place, ($func $(:: $(<$($args),*>::)? $path)*!(field_place $($rest)*)), $($binding)?));
        $crate::init!(@inner($var $pin ($($inner)* $field: ::core::todo!(),) $name $(<$($generic),*>)?) $($tail)*);
    };
    // an async function call initializing a single field
    (@inner($var:ident $pin:ident ($($inner:tt)*) $name:ident $(<$($generic:ty),*>)?)
        $(let $binding:pat = )?$func:ident $(:: $(<$($args:ty),*$(,)?>::)? $path:ident)*(.$field:ident $($rest:tt)*).await;
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
            // definetly initialize the specified field. we scope it here, to ensure no usage
            // outside of this macro.
            #[doc(hidden)]
            struct ___LocalGuard;
            // get the correct pin projection (handled by the ___PinData type)
            let $field_place = unsafe {
                <$name $(<$($generic),*>)? as $crate::place::___PinData>::___PinData::$field(
                    ::core::ptr::addr_of_mut!((*$crate::place::___PlaceInit::___as_mut_ptr(&mut $var, &|_: &$name $(<$($generic),*>)?| {})).$field),
                    &$var,
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
                // unwrap the value produced by the function immediatly, do not give access to the
                // raw InitProof. Validate using the guard, if guard would be used a second time,
                // then a move error would occur.
                result = $crate::init::InitProof::unwrap($($call)*, guard);
            }
        }
        $(let $binding = result;)?
        // create a mutable reference to the object, it can now be used, because it was initalized.
        let $field = {
            unsafe {
                &mut *::core::ptr::addr_of_mut!((*$crate::place::___PlaceInit::___as_mut_ptr(&mut $var, &|_: &$name $(<$($generic),*>)?| {})).$field)
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
        $vis:vis struct $name:ident $(<$($generic:ident),* $(,)?>)? $(where $($whr:path : $bound:ty),* $(,)?)? {
            $(
                $(#$pin:ident)?
                $(#[$attr:meta])*
                $field:ident : $type:ty
            ),*
            $(,)?
        }
    ) => {
        $(#[$struct_attr])*
        $vis struct $name $(<$($generic),*>)? $(where $($whr : $bound),*)? {
            $(
                $(#[$attr])*
                $field: $type
            ),*
        }

        const _: () = {
            $vis struct ___ThePinData;

            impl ___ThePinData {
                $(
                    pin_data!(@make_fn(($vis) $($pin)? $field: $type));
                )*
            }

            unsafe impl$(<$($generic),*>)? $crate::place::___PinData for $name$(<$($generic),*>)? {
                type ___PinData = ___ThePinData;
            }
        };
    };
    (@make_fn(($vis:vis) pin $field:ident : $type:ty)) => {
        $vis unsafe fn $field<'a, T, P: $crate::place::___PlaceInit + $crate::place::___PinnedPlace, G>(ptr: *mut T, _place: &P, guard: G) -> $crate::init::PinInitMe<'a, T, G> {
            $crate::init::PinInitMe::___new(ptr, guard)
        }
    };
    (@make_fn(($vis:vis) $field:ident : $type:ty)) => {
        $vis unsafe fn $field<'a, T,P: $crate::place::___PlaceInit, G>(ptr: *mut T, _place: &P, guard: G) -> $crate::init::InitMe<'a, T, G> {
            $crate::init::InitMe::___new(ptr, guard)
        }
    };
}
