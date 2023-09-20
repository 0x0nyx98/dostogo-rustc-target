//! Copyright © 2014–2022 Chris Morgan
//! https://github.com/chris-morgan/anymap/blob/master/COPYING
//!
//! This crate provides a safe and convenient store for one value of each type.
//!
//! Your starting point is [`Map`]. It has an example.
//!
//! # Cargo features
//!
//! This crate has two independent features, each of which provides an implementation providing
//! types `Map`, `AnyMap`, `OccupiedEntry`, `VacantEntry`, `Entry` and `RawMap`:
//!
#![cfg_attr(feature = "std", doc = " - **std** (default, *enabled* in this build):")]
#![cfg_attr(not(feature = "std"), doc = " - **std** (default, *disabled* in this build):")]
//!   an implementation using `std::collections::hash_map`, placed in the crate root
//!   (e.g. `anymap::AnyMap`).
//!
#![cfg_attr(feature = "hashbrown", doc = " - **hashbrown** (optional; *enabled* in this build):")]
#![cfg_attr(
    not(feature = "hashbrown"),
    doc = " - **hashbrown** (optional; *disabled* in this build):"
)]
//!   an implementation using `alloc` and `hashbrown::hash_map`, placed in a module `hashbrown`
//!   (e.g. `anymap::hashbrown::AnyMap`).

#![warn(missing_docs, unused_results)]
#![cfg_attr(not(feature = "std"), no_std)]

use core::convert::TryInto;
use core::hash::Hasher;

pub use crate::any::CloneAny;

mod any;

/// A hasher designed to eke a little more speed out, given `TypeId`’s known characteristics.
///
/// Specifically, this is a no-op hasher that expects to be fed a u64’s worth of
/// randomly-distributed bits. It works well for `TypeId` (eliminating start-up time, so that my
/// get_missing benchmark is ~30ns rather than ~900ns, and being a good deal faster after that, so
/// that my insert_and_get_on_260_types benchmark is ~12μs instead of ~21.5μs), but will
/// panic in debug mode and always emit zeros in release mode for any other sorts of inputs, so
/// yeah, don’t use it! 😀
#[derive(Default)]
pub struct TypeIdHasher {
    value: u64,
}

impl Hasher for TypeIdHasher {
    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        // This expects to receive exactly one 64-bit value, and there’s no realistic chance of
        // that changing, but I don’t want to depend on something that isn’t expressly part of the
        // contract for safety. But I’m OK with release builds putting everything in one bucket
        // if it *did* change (and debug builds panicking).
        debug_assert_eq!(bytes.len(), 8);
        let _ = bytes.try_into().map(|array| self.value = u64::from_ne_bytes(array));
    }

    #[inline]
    fn finish(&self) -> u64 {
        self.value
    }
}

#[cfg(any(feature = "std", feature = "hashbrown"))]
macro_rules! everything {
    ($example_init:literal, $($parent:ident)::+ $(, $entry_generics:ty)?) => {
        use core::any::{Any, TypeId};
        use core::hash::BuildHasherDefault;
        use core::marker::PhantomData;

        #[cfg(not(feature = "std"))]
        use alloc::boxed::Box;

        use ::$($parent)::+::hash_map::{self, HashMap};

        use crate::any::{Downcast, IntoBox};

        /// Raw access to the underlying `HashMap`.
        ///
        /// This alias is provided for convenience because of the ugly third generic parameter.
        pub type RawMap<A> = HashMap<TypeId, Box<A>, BuildHasherDefault<TypeIdHasher>>;

        /// A collection containing zero or one values for any given type and allowing convenient,
        /// type-safe access to those values.
        ///
        /// The type parameter `A` allows you to use a different value type; normally you will want
        /// it to be `core::any::Any` (also known as `std::any::Any`), but there are other choices:
        ///
        /// - If you want the entire map to be cloneable, use `CloneAny` instead of `Any`; with
        ///   that, you can only add types that implement `Clone` to the map.
        /// - You can add on `+ Send` or `+ Send + Sync` (e.g. `Map<dyn Any + Send>`) to add those
        ///   auto traits.
        ///
        /// Cumulatively, there are thus six forms of map:
        ///
        /// - <code>[Map]&lt;dyn [core::any::Any]&gt;</code>,
        ///   also spelled [`AnyMap`] for convenience.
        /// - <code>[Map]&lt;dyn [core::any::Any] + Send&gt;</code>
        /// - <code>[Map]&lt;dyn [core::any::Any] + Send + Sync&gt;</code>
        /// - <code>[Map]&lt;dyn [CloneAny]&gt;</code>
        /// - <code>[Map]&lt;dyn [CloneAny] + Send&gt;</code>
        /// - <code>[Map]&lt;dyn [CloneAny] + Send + Sync&gt;</code>
        ///
        /// ## Example
        ///
        /// (Here using the [`AnyMap`] convenience alias; the first line could use
        /// <code>[anymap::Map][Map]::&lt;[core::any::Any]&gt;::new()</code> instead if desired.)
        ///
        /// ```rust
        #[doc = $example_init]
        /// assert_eq!(data.get(), None::<&i32>);
        /// ```
        ///
        /// Values containing non-static references are not permitted.
        #[derive(Debug)]
        pub struct Map<A: ?Sized + Downcast = dyn Any> {
            raw: RawMap<A>,
        }

        /// The most common type of `Map`: just using `Any`; <code>[Map]&lt;dyn [Any]&gt;</code>.
        ///
        /// Why is this a separate type alias rather than a default value for `Map<A>`?
        /// `Map::new()` doesn’t seem to be happy to infer that it should go with the default
        /// value. It’s a bit sad, really. Ah well, I guess this approach will do.
        pub type AnyMap = Map<dyn Any>;
        impl<A: ?Sized + Downcast> Default for Map<A> {
            #[inline]
            fn default() -> Map<A> {
                Map::new()
            }
        }

        impl<A: ?Sized + Downcast> Map<A> {
            /// Create an empty collection.
            #[inline]
            pub fn new() -> Map<A> {
                Map {
                    raw: RawMap::with_hasher(Default::default()),
                }
            }

            /// Returns a reference to the value stored in the collection for the type `T`,
            /// if it exists.
            #[inline]
            pub fn get<T: IntoBox<A>>(&self) -> Option<&T> {
                self.raw.get(&TypeId::of::<T>())
                    .map(|any| unsafe { any.downcast_ref_unchecked::<T>() })
            }

            /// Gets the entry for the given type in the collection for in-place manipulation
            #[inline]
            pub fn entry<T: IntoBox<A>>(&mut self) -> Entry<A, T> {
                match self.raw.entry(TypeId::of::<T>()) {
                    hash_map::Entry::Occupied(e) => Entry::Occupied(OccupiedEntry {
                        inner: e,
                        type_: PhantomData,
                    }),
                    hash_map::Entry::Vacant(e) => Entry::Vacant(VacantEntry {
                        inner: e,
                        type_: PhantomData,
                    }),
                }
            }

        }

        /// A view into a single occupied location in an `Map`.
        pub struct OccupiedEntry<'a, A: ?Sized + Downcast, V: 'a> {
            inner: hash_map::OccupiedEntry<'a, TypeId, Box<A>, $($entry_generics)?>,
            type_: PhantomData<V>,
        }

        /// A view into a single empty location in an `Map`.
        pub struct VacantEntry<'a, A: ?Sized + Downcast, V: 'a> {
            inner: hash_map::VacantEntry<'a, TypeId, Box<A>, $($entry_generics)?>,
            type_: PhantomData<V>,
        }

        /// A view into a single location in an `Map`, which may be vacant or occupied.
        pub enum Entry<'a, A: ?Sized + Downcast, V: 'a> {
            /// An occupied Entry
            Occupied(OccupiedEntry<'a, A, V>),
            /// A vacant Entry
            Vacant(VacantEntry<'a, A, V>),
        }

        impl<'a, A: ?Sized + Downcast, V: IntoBox<A>> Entry<'a, A, V> {


            /// Ensures a value is in the entry by inserting the result of the default function if
            /// empty, and returns a mutable reference to the value in the entry.
            #[inline]
            pub fn or_insert_with<F: FnOnce() -> V>(self, default: F) -> &'a mut V {
                match self {
                    Entry::Occupied(inner) => inner.into_mut(),
                    Entry::Vacant(inner) => inner.insert(default()),
                }
            }
        }

        impl<'a, A: ?Sized + Downcast, V: IntoBox<A>> OccupiedEntry<'a, A, V> {
            /// Converts the OccupiedEntry into a mutable reference to the value in the entry
            /// with a lifetime bound to the collection itself
            #[inline]
            pub fn into_mut(self) -> &'a mut V {
                unsafe { self.inner.into_mut().downcast_mut_unchecked() }
            }
        }

        impl<'a, A: ?Sized + Downcast, V: IntoBox<A>> VacantEntry<'a, A, V> {
            /// Sets the value of the entry with the VacantEntry's key,
            /// and returns a mutable reference to it
            #[inline]
            pub fn insert(self, value: V) -> &'a mut V {
                unsafe { self.inner.insert(value.into_box()).downcast_mut_unchecked() }
            }
        }

        #[cfg(test)]
        mod tests {
            use crate::CloneAny;
            use super::*;

            #[derive(Clone, Debug, PartialEq)] struct A(i32);
            #[derive(Clone, Debug, PartialEq)] struct B(i32);
            #[derive(Clone, Debug, PartialEq)] struct C(i32);
            #[derive(Clone, Debug, PartialEq)] struct D(i32);
            #[derive(Clone, Debug, PartialEq)] struct E(i32);
            #[derive(Clone, Debug, PartialEq)] struct F(i32);
            #[derive(Clone, Debug, PartialEq)] struct J(i32);

            #[test]
            fn test_varieties() {
                fn assert_send<T: Send>() { }
                fn assert_sync<T: Sync>() { }
                fn assert_debug<T: ::core::fmt::Debug>() { }
                assert_send::<Map<dyn Any + Send>>();
                assert_send::<Map<dyn Any + Send + Sync>>();
                assert_sync::<Map<dyn Any + Send + Sync>>();
                assert_debug::<Map<dyn Any>>();
                assert_debug::<Map<dyn Any + Send>>();
                assert_debug::<Map<dyn Any + Send + Sync>>();
                assert_send::<Map<dyn CloneAny + Send>>();
                assert_send::<Map<dyn CloneAny + Send + Sync>>();
                assert_sync::<Map<dyn CloneAny + Send + Sync>>();
                assert_debug::<Map<dyn CloneAny>>();
                assert_debug::<Map<dyn CloneAny + Send>>();
                assert_debug::<Map<dyn CloneAny + Send + Sync>>();
            }
        }
    };
}

#[test]
fn type_id_hasher() {
    use core::any::TypeId;
    use core::hash::Hash;
    fn verify_hashing_with(type_id: TypeId) {
        let mut hasher = TypeIdHasher::default();
        type_id.hash(&mut hasher);
        // SAFETY: u64 is valid for all bit patterns.
        let _ = hasher.finish();
    }
    // Pick a variety of types, just to demonstrate it’s all sane. Normal, zero-sized, unsized, &c.
    verify_hashing_with(TypeId::of::<usize>());
    verify_hashing_with(TypeId::of::<()>());
    verify_hashing_with(TypeId::of::<str>());
    verify_hashing_with(TypeId::of::<&str>());
    verify_hashing_with(TypeId::of::<Vec<u8>>());
}

#[cfg(feature = "std")]
everything!("let mut data = anymap::AnyMap::new();", std::collections);
