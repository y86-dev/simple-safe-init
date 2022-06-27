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
                init!(@inner(var _is_pinned () $struct $(<$($generic),*>)?) $($tail)*);
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
    (@inner($var:ident $pin:ident ($($inner:tt)*) $name:ident $(<$($generic:ty),*>)?)) => {
        #[allow(unreachable_code)]
        let ____check_all_init = || {
            let _struct: $name $(<$($generic),*>)? = $name {
                $($inner)*
            };
        };
    };
    (@inner($var:ident $pin:ident ($($inner:tt)*) $name:ident $(<$($generic:ty),*>)?) .$field:ident = $val:expr; $($tail:tt)*) => {
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
        init!(@inner($var $pin ($($inner)* $field: ::core::todo!(),) $name $(<$($generic),*>)?) $($tail)*);
    };
    (@inner($var:ident $pin:ident ($($inner:tt)*) $name:ident $(<$($generic:ty),*>)?) $(.let $binding:pat = )?$func:ident $(:: $(<$($args:ty),*$(,)?>::)? $path:ident)*(.$field:ident $($rest:tt)*); $($tail:tt)*) => {
        let result;
        {
            struct ___LocalGuard;
            let value = unsafe {
                // SAFETY: we are
                // - not inspecting the pointee (might be uninit)
                // - not moving the value (might be pinned)
                // - not expanding any uncontrollable macro input
                <$name $(<$($generic),*>)? as $crate::place::___PinData>::___PinData::$field(
                    ::core::ptr::addr_of_mut!((*$crate::place::___PlaceInit::___as_mut_ptr(&mut $var, &|_: &$name $(<$($generic),*>)?| {})).$field),
                    &$var,
                    ___LocalGuard,
                )
            };
            let guard = ___LocalGuard;
            {
                struct ___LocalGuard;
                result = $crate::init::InitProof::unwrap($func $(:: $(<$($args),*>::)? $path)*(value $($rest)*), guard);
            }
        }
        $(let $binding = result;)?
        let $field = {
            unsafe {
                &mut *::core::ptr::addr_of_mut!((*$crate::place::___PlaceInit::___as_mut_ptr(&mut $var, &|_: &$name $(<$($generic),*>)?| {})).$field)
            }
        };
        #[allow(unused_variables)]
        let $field = $field;
        init!(@inner($var $pin ($($inner)* $field: ::core::todo!(),) $name $(<$($generic),*>)?) $($tail)*);
    };
    (@inner($var:ident $pin:ident ($($inner:tt)*) $name:ident $(<$($generic:ty),*>)?) $(.let $binding:pat = )?unsafe { $func:ident $(:: $(<$($args:ty),*$(,)?>::)? $path:ident)*(.$field:ident $($rest:tt)*) }; $($tail:tt)*) => {
        let result;
        {
            struct ___LocalGuard;
            let value = unsafe {
                // SAFETY: we are
                // - not inspecting the pointee (might be uninit)
                // - not moving the value (might be pinned)
                <$name $(<$($generic),*>)? as $crate::place::___PinData>::___PinData::$field(
                    ::core::ptr::addr_of_mut!((*$crate::place::___PlaceInit::___as_mut_ptr(&mut $var, &|_: &$name $(<$($generic),*>)?| {})).$field),
                    &$var,
                    ___LocalGuard,
                )
            };
            let guard = ___LocalGuard;
            {
                struct ___LocalGuard;
                result = $crate::init::InitProof::unwrap(unsafe { $func $(:: $(<$($args),*>::)? $path)*(value $($rest)*) }, guard);
            }
        }
        $(let $binding = result;)?
        let $field = {
            unsafe {
                &mut *::core::ptr::addr_of_mut!((*$crate::place::___PlaceInit::___as_mut_ptr(&mut $var, &|_: &$name $(<$($generic),*>)?| {})).$field)
            }
        };
        #[allow(unused_variables)]
        let $field = $field;
        init!(@inner($var $pin ($($inner)* $field: ::core::todo!(),) $name $(<$($generic),*>)?) $($tail)*);
    };
    (@inner($var:ident $pin:ident ($($inner:tt)*) $($name:tt)*) $st:stmt; $($tail:tt)*) => {
        $st
        init!(@inner($var $pin ($($inner)*) $($name)*) $($tail)*);
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
