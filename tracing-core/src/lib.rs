//! Core primitives for `tracing`.
//!
//! [`tracing`] is a framework for instrumenting Rust programs to collect
//! structured, event-based diagnostic information. This crate defines the core
//! primitives of `tracing`.
//!
//! This crate provides:
//!
//! * [`span::Id`] identifies a span within the execution of a program.
//!
//! * [`Event`] represents a single event within a trace.
//!
//! * [`Subscriber`], the trait implemented to collect trace data.
//!
//! * [`Metadata`] and [`Callsite`] provide information describing spans and
//!   `Event`s.
//!
//! * [`Field`], [`FieldSet`], [`Value`], and [`ValueSet`] represent the
//!   structured data attached to a span.
//!
//! * [`Dispatch`] allows spans and events to be dispatched to `Subscriber`s.
//!
//! In addition, it defines the global callsite registry and per-thread current
//! dispatcher which other components of the tracing system rely on.
//!
//! ## Usage
//!
//! Application authors will typically not use this crate directly. Instead,
//! they will use the [`tracing`] crate, which provides a much more
//! fully-featured API. However, this crate's API will change very infrequently,
//! so it may be used when dependencies must be very stable.
//!
//! `Subscriber` implementations may depend on `tracing-core` rather than
//! `tracing`, as the additional APIs provided by `tracing` are primarily useful
//! for instrumenting libraries and applications, and are generally not
//! necessary for `Subscriber` implementations.
//!
//! The [`tokio-rs/tracing`] repository contains less stable crates designed to
//! be used with the `tracing` ecosystem. It includes a collection of
//! `Subscriber` implementations, as well as utility and adapter crates.
//!
//! ### Crate Feature Flags
//!
//! The following crate feature flags are available:
//!
//! * `std`: Depend on the Rust standard library (enabled by default).
//!
//!   `no_std` users may disable this feature with `default-features = false`:
//!
//!   ```toml
//!   [dependencies]
//!   tracing-core = { version = "0.1.7", default-features = false }
//!   ```
//!   **Note**:`tracing-core`'s `no_std` support requires `liballoc`.
//!
//! [`span::Id`]: span/struct.Id.html
//! [`Event`]: event/struct.Event.html
//! [`Subscriber`]: subscriber/trait.Subscriber.html
//! [`Metadata`]: metadata/struct.Metadata.html
//! [`Callsite`]: callsite/trait.Callsite.html
//! [`Field`]: field/struct.Field.html
//! [`FieldSet`]: field/struct.FieldSet.html
//! [`Value`]: field/trait.Value.html
//! [`ValueSet`]: field/struct.ValueSet.html
//! [`Dispatch`]: dispatcher/struct.Dispatch.html
//! [`tokio-rs/tracing`]: https://github.com/tokio-rs/tracing
//! [`tracing`]: https://crates.io/crates/tracing
#![doc(html_root_url = "https://docs.rs/tracing-core/0.1.7")]
#![cfg_attr(not(feature = "std"), no_std)]
#![warn(
    missing_debug_implementations,
    missing_docs,
    rust_2018_idioms,
    unreachable_pub,
    bad_style,
    const_err,
    dead_code,
    improper_ctypes,
    legacy_directory_ownership,
    non_shorthand_field_patterns,
    no_mangle_generic_items,
    overflowing_literals,
    path_statements,
    patterns_in_fns_without_body,
    plugin_as_library,
    private_in_public,
    safe_extern_statics,
    unconditional_recursion,
    unused,
    unused_allocation,
    unused_comparisons,
    unused_parens,
    while_true
)]
#[cfg(not(feature = "std"))]
extern crate alloc;

/// Statically constructs an [`Identifier`] for the provided [`Callsite`].
///
/// This may be used in contexts, such as static initializers, where the
/// [`Callsite::id`] function is not currently usable.
///
/// For example:
/// ```rust
/// # #[macro_use]
/// # extern crate tracing_core;
/// use tracing_core::callsite;
/// # use tracing_core::{Metadata, subscriber::Interest};
/// # fn main() {
/// pub struct MyCallsite {
///    // ...
/// }
/// impl callsite::Callsite for MyCallsite {
/// # fn set_interest(&self, _: Interest) { unimplemented!() }
/// # fn metadata(&self) -> &Metadata { unimplemented!() }
///     // ...
/// }
///
/// static CALLSITE: MyCallsite = MyCallsite {
///     // ...
/// };
///
/// static CALLSITE_ID: callsite::Identifier = identify_callsite!(&CALLSITE);
/// # }
/// ```
///
/// [`Identifier`]: callsite/struct.Identifier.html
/// [`Callsite`]: callsite/trait.Callsite.html
/// [`Callsite`]: callsite/trait.Callsite.html#method.id
#[macro_export]
macro_rules! identify_callsite {
    ($callsite:expr) => {
        $crate::callsite::Identifier($callsite)
    };
}

/// Statically constructs new span [metadata].
///
/// /// For example:
/// ```rust
/// # #[macro_use]
/// # extern crate tracing_core;
/// # use tracing_core::{callsite::Callsite, subscriber::Interest};
/// use tracing_core::metadata::{Kind, Level, Metadata};
/// # fn main() {
/// # pub struct MyCallsite { }
/// # impl Callsite for MyCallsite {
/// # fn set_interest(&self, _: Interest) { unimplemented!() }
/// # fn metadata(&self) -> &Metadata { unimplemented!() }
/// # }
/// #
/// static FOO_CALLSITE: MyCallsite = MyCallsite {
///     // ...
/// };
///
/// static FOO_METADATA: Metadata = metadata!{
///     name: "foo",
///     target: module_path!(),
///     level: Level::DEBUG,
///     fields: &["bar", "baz"],
///     callsite: &FOO_CALLSITE,
///     kind: Kind::SPAN,
/// };
/// # }
/// ```
///
/// [metadata]: metadata/struct.Metadata.html
/// [`Metadata::new`]: metadata/struct.Metadata.html#method.new
#[macro_export]
macro_rules! metadata {
    (
        name: $name:expr,
        target: $target:expr,
        level: $level:expr,
        fields: $fields:expr,
        callsite: $callsite:expr,
        kind: $kind:expr
    ) => {
        metadata! {
            name: $name,
            target: $target,
            level: $level,
            fields: $fields,
            callsite: $callsite,
            kind: $kind,
        }
    };
    (
        name: $name:expr,
        target: $target:expr,
        level: $level:expr,
        fields: $fields:expr,
        callsite: $callsite:expr,
        kind: $kind:expr,
    ) => {
        $crate::metadata::Metadata::new(
            $name,
            $target,
            $level,
            Some(file!()),
            Some(line!()),
            Some(module_path!()),
            $crate::field::FieldSet::new($fields, $crate::identify_callsite!($callsite)),
            $kind,
        )
    };
}

// std uses lazy_static from crates.io
#[cfg(feature = "std")]
#[macro_use]
extern crate lazy_static;

// no_std uses vendored version of lazy_static 1.4.0 (4216696) with spin
// This can conflict when included in a project already using std lazy_static
// Remove this module when cargo enables specifying dependencies for no_std
#[cfg(not(feature = "std"))]
#[macro_use]
mod lazy_static;

// Trimmed-down vendored version of spin 0.5.2 (0387621)
// Dependency of no_std lazy_static, not required in a std build
#[cfg(not(feature = "std"))]
pub(crate) mod spin;

#[cfg(not(feature = "std"))]
#[doc(hidden)]
pub use self::spin::Once;

pub mod callsite;
pub mod dispatcher;
pub mod event;
pub mod field;
pub mod metadata;
mod parent;
pub mod span;
pub(crate) mod stdlib;
pub mod subscriber;

#[doc(inline)]
pub use self::{
    callsite::Callsite,
    dispatcher::Dispatch,
    event::Event,
    field::Field,
    metadata::{Level, Metadata},
    subscriber::Subscriber,
};

pub use self::{metadata::Kind, subscriber::Interest};

mod sealed {
    pub trait Sealed {}
}
