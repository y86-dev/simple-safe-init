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
    // initialize an arbitrary expression manually (init each field).
    ($var:expr => $struct:ident $(<$($generic:ty),*>)? { $($tail:tt)* }) => {
        match $var {
            mut var => {
                fn no_warn<___T>(_: &mut ___T) {}
                no_warn(&mut var);
                $crate::init!(@@inner(var, _is_pinned, (), ($struct $(<$($generic),*>)?)) $($tail)*);
                unsafe {
                    // SAFETY: The pointee of `var` has been fully initialized, if this part is
                    // reachable and no compile error exist.
                    $crate::place::PartialInitPlace::___init(var)
                }
            }
        }
    };
    // initialize a specific AllocablePlace manually (init each field).
    (@$var:ty => $struct:ident $(<$($generic:ty),*>)? { $($tail:tt)* }) => {
        <$var as $crate::place::AllocablePlace>::allocate().map(move |mut var| {
            fn no_warn<___T>(_: &mut ___T) {}
            no_warn(&mut var);
            $crate::init!(@@inner(var, _is_pinned, (), ($struct $(<$($generic),*>)?)) $($tail)*);
            unsafe {
                // SAFETY: The pointee of `var` has been fully initialized, if this part is
                // reachable and no compile error exist.
                $crate::place::PartialInitPlace::___init(var)
            }
        })
    };
    // initialize a specific AllocablePlace using a single macro.
    ($func:ident $(:: $(<$($args:ty),*$(,)?>::)? $path:ident)*!(@@$var:ty $(, $($rest:tt)*)?)) => {
        <$var as $crate::place::AllocablePlace>::allocate().map(move |var| {
            $crate::init!(@@fully_init(var, ($func $(:: $(<$($args),*>::)? $path)*!) $(, $($rest)*)?))
        })
    };
    // initialize a specific AllocablePlace using a single function.
    ($func:ident $(:: $(<$($args:ty),*$(,)?>::)? $path:ident)*(@@$var:ty $(, $($rest:tt)*)?)) => {
        <$var as $crate::place::AllocablePlace>::allocate().map(move |var| {
            $crate::init!(@@fully_init(var, ($func $(:: $(<$($args),*>::)? $path)*) $(, $($rest)*)?))
        })
    };
    // initialize an arbitrary expression using a single macro.
    ($func:ident $(:: $(<$($args:ty),*$(,)?>::)? $path:ident)*!($var:expr $(, $($rest:tt)*)?)) => {
        $crate::init!(@@fully_init($var, ($func $(:: $(<$($args),*>::)? $path)*!) $(, $($rest)*)?))
    };
    // initialize an arbitrary expression using a single function.
    ($func:ident $(:: $(<$($args:ty),*$(,)?>::)? $path:ident)*($var:expr $(, $($rest:tt)*)?)) => {
        $crate::init!(@@fully_init($var, ($func $(:: $(<$($args),*>::)? $path)*) $(, $($rest)*)?))
    };

    /*
     * ##############
     * internal rules
     * ##############
     */

    // when there is no input left, construct a struct initializer with all of the fields
    // mentioned. If one is missing or a duplicate, the compiler will complain.
    // We do this inside of a closure, because we do not want to really create this struct. Also,
    // the values of the fields are `conjure()` so we never actually produce a value.
    (@@inner($var:ident, $pin:ident, ($($inner:tt)*), ($name:ident $(<$($generic:ty),*>)?))) => {
        #[allow(unreachable_code)]
        let ____check_all_init = || {
            let _struct: $name $(<$($generic),*>)? = $name {
                $($inner)*
            };
        };
    };
    // a normal assignment, use raw pointers to set the value.
    (@@inner($var:ident, $pin:ident, ($($inner:tt)*), ($name:ident $(<$($generic:ty),*>)?))
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
        $crate::init!(@@inner($var, $pin, ($($inner)* $field: $crate::conjure(),), ($name $(<$($generic),*>)?)) $($tail)*);
    };
    // a function call initializing a single field, we cannot use the `path` meta-variable type,
    // because `(` is not allowed after that :(
    (@@inner($var:ident, $pin:ident, ($($inner:tt)*), ($name:ident $(<$($generic:ty),*>)?))
        $(~let $binding:pat = )?$func:ident $(:: $(<$($args:ty),*$(,)?>::)? $path:ident)*(.$field:ident $($rest:tt)*);
        $($tail:tt)*
    ) => {
        $crate::init!(@@init_call($var, $name $(<$($generic),*>)?, $field, field_place, ($func $(:: $(<$($args),*>::)? $path)*(field_place $($rest)*)), $($binding)?));
        $crate::init!(@@inner($var, $pin, ($($inner)* $field: $crate::conjure(),), ($name $(<$($generic),*>)?)) $($tail)*);
    };
    // an unsafe function initializing a single field.
    (@@inner($var:ident, $pin:ident, ($($inner:tt)*), ($name:ident $(<$($generic:ty),*>)?))
        $(~let $binding:pat = )?unsafe { $func:ident $(:: $(<$($args:ty),*$(,)?>::)? $path:ident)*(.$field:ident $($rest:tt)*) };
        $($tail:tt)*
    ) => {
        $crate::init!(@@init_call($var, $name $(<$($generic),*>)?, $field, field_place, (unsafe { $func $(:: $(<$($args),*>::)? $path)*(field_place $($rest)*) }), $($binding)?));
        $crate::init!(@@inner($var $pin ($($inner)* $field: $crate::conjure(),) $name $(<$($generic),*>)?) $($tail)*);
    };
    // a macro call initializing a single field
    (@@inner($var:ident, $pin:ident, ($($inner:tt)*), ($name:ident $(<$($generic:ty),*>)?))
        $(~let $binding:pat = )?$func:ident $(:: $(<$($args:ty),*$(,)?>::)? $path:ident)*!(.$field:ident $($rest:tt)*);
        $($tail:tt)*
    ) => {
        $crate::init!(@@init_call($var, $name $(<$($generic),*>)?, $field, field_place, ($func $(:: $(<$($args),*>::)? $path)*!(field_place $($rest)*)), $($binding)?));
        $crate::init!(@@inner($var, $pin, ($($inner)* $field: $crate::conjure(),), ($name $(<$($generic),*>)?)) $($tail)*);
    };
    // an async function call initializing a single field
    (@@inner($var:ident, $pin:ident, ($($inner:tt)*), ($name:ident $(<$($generic:ty),*>)?))
        $(~let $binding:pat = )?$func:ident $(:: $(<$($args:ty),*$(,)?>::)? $path:ident)*(.$field:ident $($rest:tt)*).await;
        $($tail:tt)*
    ) => {
        $crate::init!(@@init_call($var, $name $(<$($generic),*>)?, $field, field_place, ($func $(:: $(<$($args),*>::)? $path)*(field_place $($rest)*).await), $($binding)?));
        $crate::init!(@@inner($var, $pin, ($($inner)* $field: $crate::conjure(),), ($name $(<$($generic),*>)?)) $($tail)*);
    };
    // a normal statement that will be executed as-is.
    (@@inner($var:ident, $pin:ident, ($($inner:tt)*), ($($name:tt)*))
        $st:stmt;
        $($tail:tt)*
    ) => {
        $st
        $crate::init!(@@inner($var, $pin, ($($inner)*), ($($name)*)) $($tail)*);
    };
    // generalized function/macro call helper (manual)
    (@@init_call($var:ident, $name:ident $(<$($generic:ty),*>)?, $field:ident, $field_place:ident, ($($call:tt)*), $($binding:pat)?)) => {
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
                result = $crate::InitProof::unwrap($($call)*, guard);
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
    // generalized single function/macro init helper
    (@@fully_init($var:expr, ($($init:tt)*)$(, $($rest:tt)*)?)) => {
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
                        let () = $crate::InitProof::unwrap($($init)*(value $(, $($rest)*)?), guard);
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
                    $crate::pin_data!(@@make_fn(($fvis) $($pin)? $field: $type));
                )*
            }

            unsafe impl$(<$($($life),+ ,)? $($generic $(: ?$sized)?),*>)? $crate::place::___PinData for $name$(<$($($life),+ ,)? $($generic),*>)? {
                type ___PinData = ___ThePinData;
            }
        };
    };
    (@@make_fn(($vis:vis) pin $field:ident : $type:ty)) => {
        $vis unsafe fn $field<'a, T, P: $crate::place::PinnedPlace, G>(ptr: *mut T, _place: Option<&P>, guard: G) -> $crate::PinInitMe<'a, T, G> {
            unsafe { $crate::PinInitMe::___new(ptr, guard) }
        }
    };
    (@@make_fn(($vis:vis) $field:ident : $type:ty)) => {
        $vis unsafe fn $field<'a, T,P: $crate::place::PartialInitPlace, G>(ptr: *mut T, _place: Option<&P>, guard: G) -> $crate::InitMe<'a, T, G> {
            unsafe { $crate::InitMe::___new(ptr, guard) }
        }
    };
}

#[macro_export]
macro_rules! stack_init {
    ($var:ident: $typ:ty => { $($tail:tt)* }) => {
        let mut $var: ::core::mem::MaybeUninit<$typ> = ::core::mem::MaybeUninit::uninit();
        {
            struct ___LocalGuard;
            let tmp = unsafe {
                // SAFETY: we never move out of $var and shadow it at the end so
                // no one can move out of it.
                <$crate::PinInitMe<$typ, ___LocalGuard> as $crate::InitPointer<$typ, ___LocalGuard>>::___new(
                    ::core::mem::MaybeUninit::as_mut_ptr(&mut $var),
                    ___LocalGuard
                )
            };
            let guard = ___LocalGuard;
            {
                struct ___LocalGuard;
                let () = $crate::InitProof::unwrap(
                    $crate::init! { tmp => $typ { $($tail)* }},
                    guard
                );
            }
        }
        // SAFETY: variable is shadowed, so it cannot be moved out of.
        let mut $var = unsafe {
            ::core::pin::Pin::new_unchecked(::core::mem::MaybeUninit::assume_init_mut(&mut $var))
        };
    };
    ($var:ident: $typ:ty => ( $($tail:tt)* )) => {
        let mut $var: ::core::mem::MaybeUninit<$typ> = ::core::mem::MaybeUninit::uninit();
        {
            struct ___LocalGuard;
            let $var = unsafe {
                // SAFETY: we never move out of $var and shadow it at the end so
                // no one can move out of it.
                <$crate::PinInitMe<$typ, ___LocalGuard> as $crate::InitPointer<$typ, ___LocalGuard>>::___new(
                    ::core::mem::MaybeUninit::as_mut_ptr(&mut $var),
                    ___LocalGuard
                )
            };
            let guard = ___LocalGuard;
            {
                struct ___LocalGuard;
                let () = $crate::InitProof::unwrap(
                    $crate::init!($($tail)*),
                    guard
                );
            }
        }
        // SAFETY: variable is shadowed, so it cannot be moved out of.
        let mut $var = unsafe {
            ::core::pin::Pin::new_unchecked(::core::mem::MaybeUninit::assume_init_mut(&mut $var))
        };
    };
}
