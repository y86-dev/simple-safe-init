#![allow(dead_code)]

/// ```rust,compile_fail
/// use simple_safe_init::*;
/// use core::mem::MaybeUninit;
///
/// #[derive(Debug)]
/// struct Foo {
///     a: u32,
///     b: u64,
/// }
/// let foo = MaybeUninit::uninit();
/// let foo = init! { foo => Foo {
///     .a = 42;
/// }};
/// println!("{:?}", foo);
/// ```
///
fn prevent_missing_field() {}

/// ```rust,compile_fail
/// use simple_safe_init::*;
/// use core::mem::MaybeUninit;
///
/// #[derive(Debug)]
/// struct Foo {
///     a: u32,
///     b: u64,
/// }
/// let foo = MaybeUninit::uninit();
/// let foo = init! { foo => Foo {
///     .a = 42;
///     .b = 30;
///     .a = 2;
/// }};
/// println!("{:?}", foo);
/// ```
///
fn prevent_duplicate() {}

/// ```rust
/// use simple_safe_init::*;
/// use core::mem::MaybeUninit;
///
/// #[derive(Debug)]
/// struct Foo {
///     a: u32,
///     b: u64,
/// }
/// let foo = MaybeUninit::uninit();
/// let foo = init! { foo => Foo {
///     .a = 42;
///     .b = 30;
/// }};
/// println!("{:?}", foo);
/// ```
///
fn basic() {}

/// ```rust
/// use simple_safe_init::*;
/// use core::mem::MaybeUninit;
///
/// #[derive(Debug)]
/// struct Foo {
///     a: u32,
///     b: u64,
/// }
///
/// impl Foo {
///     pub fn init_foo<G: Guard>(foo: InitMe<'_, Self, G>) -> InitProof<(), G> {
///         init! { foo => Foo {
///             .a = 42;
///             .b = 36;
///         }}
///     }
/// }
/// let foo = MaybeUninit::uninit();
/// let foo = init!(Foo::init_foo(foo));
/// println!("{:?}", foo);
/// let foo = MaybeUninit::uninit();
/// let foo = init!(Foo::init_foo(foo,));
/// println!("{:?}", foo);
/// ```
///
fn delegate() {}

/// ```rust,compile_fail
/// use simple_safe_init::*;
/// use core::mem::MaybeUninit;
///
/// #[derive(Debug)]
/// struct Foo {
///     a: u32,
///     b: u64,
/// }
///
/// impl Foo {
///     pub fn init_foo<G: Guard>(foo: InitMe<'_, Self, G>, dbg: &str) -> InitProof<(), G> {
///         println!("{}", dbg);
///         init! { foo => Foo {
///             .a = 42;
///             .b = 36;
///         }}
///     }
/// }
///
/// let foo = MaybeUninit::uninit();
/// let foo = init!(Foo::init_foo(foo));
/// println!("{:?}", foo);
/// let foo = MaybeUninit::uninit();
/// let foo = init!(Foo::init_foo(foo,));
/// println!("{:?}", foo);
/// ```
///
/// ```rust,compile_fail
/// use simple_safe_init::*;
/// use core::mem::MaybeUninit;
///
/// #[derive(Debug)]
/// struct Foo {
///     a: u32,
///     b: u64,
/// }
///
/// impl Foo {
///     pub fn init_foo<G: Guard>(foo: InitMe<'_, Self, G>) -> InitProof<(), G> {
///         init! { foo => Foo {
///             .a = 42;
///             .b = 36;
///         }}
///     }
/// }
///
/// let foo = MaybeUninit::uninit();
/// let foo = init!(Foo::init_foo(foo, "first"));
/// println!("{:?}", foo);
/// let foo = MaybeUninit::uninit();
/// let foo = init!(Foo::init_foo(foo, "second"));
/// println!("{:?}", foo);
/// ```
///
fn bad_delegate() {}
