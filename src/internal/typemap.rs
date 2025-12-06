#![allow(dead_code)]

use std::{
    any::{Any, TypeId},
    boxed::Box,
    collections::{hash_map, HashMap},
    hash::{BuildHasherDefault, Hasher},
    marker::PhantomData,
};

use self::downcast::Downcast;
#[allow(unused_imports)]
pub use self::downcast::{CloneAny, IntoBox};

mod downcast;

/// Raw access to the underlying `HashMap`.
pub type RawMap<Ty> = HashMap<TypeMapKey, Box<Ty>, BuildHasherDefault<TypeIdHasher>>;

/// A keyed TypeMap, for storing disjointed sets of types.
///
/// This collection inherits the performance characteristics
/// of the underlying `HashMap`, namely ~O(1) lookups,
/// inserts and fetches.
#[derive(Debug)]
pub struct TypeMap<Ty: ?Sized + Downcast = dyn Any> {
    raw: RawMap<Ty>,
}

// #[derive(Clone)] would want Ty to implement Clone, but in
// reality only Box<Ty> can.
impl<Ty: ?Sized + Downcast> Clone for TypeMap<Ty>
where
    Box<Ty>: Clone,
{
    #[inline]
    fn clone(&self) -> TypeMap<Ty> {
        TypeMap {
            raw: self.raw.clone(),
        }
    }
}

impl<Ty: ?Sized + Downcast> Default for TypeMap<Ty> {
    #[inline]
    fn default() -> TypeMap<Ty> {
        TypeMap::new()
    }
}

impl<Ty: ?Sized + Downcast> TypeMap<Ty> {
    /// Create an empty collection.
    #[inline]
    pub fn new() -> TypeMap<Ty> {
        TypeMap {
            raw: RawMap::with_hasher(Default::default()),
        }
    }

    /// Creates an empty collection with the given initial
    /// capacity.
    #[inline]
    pub fn with_capacity(capacity: usize) -> TypeMap<Ty> {
        TypeMap {
            raw: RawMap::with_capacity_and_hasher(capacity, Default::default()),
        }
    }

    /// Returns the number of elements the collection can
    /// hold without reallocating.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.raw.capacity()
    }

    /// Reserves capacity for at least `additional` more
    /// elements to be inserted in the collection. The
    /// collection may reserve more space to avoid
    /// frequent reallocations.
    ///
    /// # Panics
    ///
    /// Panics if the new allocation size overflows `usize`.
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.raw.reserve(additional)
    }

    /// Shrinks the capacity of the collection as much as
    /// possible. It will drop down as much as possible
    /// while maintaining the internal rules
    /// and possibly leaving some space in accordance with
    /// the resize policy.
    #[inline]
    pub fn shrink_to_fit(&mut self) {
        self.raw.shrink_to_fit()
    }

    /// Returns the number of items in the collection.
    #[inline]
    pub fn len(&self) -> usize {
        self.raw.len()
    }

    /// Returns true if there are no items in the
    /// collection.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.raw.is_empty()
    }

    /// Removes all items from the collection. Keeps the
    /// allocated memory for reuse.
    #[inline]
    pub fn clear(&mut self) {
        self.raw.clear()
    }

    /// Returns a reference to the value stored in the
    /// collection for the type `T`, if it exists.
    #[inline]
    pub fn get<V>(&self) -> Option<&V>
    where
        V: IntoBox<Ty>,
    {
        self.raw
            .get(&Self::key_type_id::<V>())
            .map(|ty| unsafe { ty.downcast_ref_unchecked::<V>() })
    }

    /// Returns a mutable reference to the value stored in
    /// the collection for the type `T`, if it exists.
    #[inline]
    pub fn get_mut<V>(&mut self) -> Option<&mut V>
    where
        V: IntoBox<Ty>,
    {
        self.raw
            .get_mut(&Self::key_type_id::<V>())
            .map(|ty| unsafe { ty.downcast_mut_unchecked::<V>() })
    }

    /// Sets the value stored in the collection for the type
    /// `T`. If the collection already had a value of
    /// type `T`, that value is returned. Otherwise,
    /// `None` is returned.
    #[inline]
    pub fn insert<V>(&mut self, value: V) -> Option<V>
    where
        V: IntoBox<Ty>,
    {
        self.raw
            .insert(Self::key_type_id::<V>(), value.into_box())
            .map(|ty| unsafe { *ty.downcast_unchecked::<V>() })
    }

    /// Removes the `T` value from the collection,
    /// returning it if there was one or `None` if there was
    /// not.
    #[inline]
    pub fn remove<V>(&mut self) -> Option<V>
    where
        V: IntoBox<Ty>,
    {
        self.raw
            .remove(&Self::key_type_id::<V>())
            .map(|ty| *unsafe { ty.downcast_unchecked::<V>() })
    }

    /// Returns true if the collection contains a value of
    /// type `T`.
    #[inline]
    pub fn contains<V>(&self) -> bool
    where
        V: IntoBox<Ty>,
    {
        self.raw.contains_key(&Self::key_type_id::<V>())
    }

    /// Gets the entry for the given type in the collection
    /// for in-place manipulation
    #[inline]
    pub fn entry<V>(&mut self) -> Entry<'_, Ty, V>
    where
        V: IntoBox<Ty>,
    {
        match self.raw.entry(Self::key_type_id::<V>()) {
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

    /// Get access to the raw hash map that backs this.
    ///
    /// This will seldom be useful, but it’s conceivable
    /// that you could wish to iterate over all the
    /// items in the collection, and this lets you do that.
    #[inline]
    pub fn as_raw(&self) -> &RawMap<Ty> {
        &self.raw
    }

    /// Get mutable access to the raw hash map that backs
    /// this.
    ///
    /// This will seldom be useful, but it’s conceivable
    /// that you could wish to iterate over all the
    /// items in the collection mutably, or drain or
    /// something, or *possibly* even batch insert, and
    /// this lets you do that.
    ///
    /// # Safety
    ///
    /// If you insert any values to the raw map, the key (a
    /// `TypeId`) must match the value’s type, or
    /// *undefined behaviour* will occur when you access
    /// those values.
    ///
    /// (*Removing* entries is perfectly safe.)
    #[inline]
    pub unsafe fn as_raw_mut(&mut self) -> &mut RawMap<Ty> {
        &mut self.raw
    }

    /// Convert this into the raw hash map that backs this.
    ///
    /// This will seldom be useful, but it’s conceivable
    /// that you could wish to consume all the items in
    /// the collection and do *something* with some or all
    /// of them, and this lets you do that, without the
    /// `unsafe` that `.as_raw_mut().drain()` would require.
    #[inline]
    pub fn into_raw(self) -> RawMap<Ty> {
        self.raw
    }

    fn key_type_id<T>() -> TypeMapKey
    where
        T: 'static,
    {
        TypeMapKey::with_typeid(TypeId::of::<T>())
    }
}

/// A view into a single occupied location in an `Map`.
pub struct OccupiedEntry<'a, Ty: ?Sized + Downcast, V: 'a> {
    inner: hash_map::OccupiedEntry<'a, TypeMapKey, Box<Ty>>,
    type_: PhantomData<V>,
}

/// A view into a single empty location in an `Map`.
pub struct VacantEntry<'a, Ty: ?Sized + Downcast, V: 'a> {
    inner: hash_map::VacantEntry<'a, TypeMapKey, Box<Ty>>,
    type_: PhantomData<V>,
}

/// A view into a single location in an `Map`, which may be
/// vacant or occupied.
pub enum Entry<'a, Ty: ?Sized + Downcast, V> {
    /// An occupied Entry
    Occupied(OccupiedEntry<'a, Ty, V>),
    /// A vacant Entry
    Vacant(VacantEntry<'a, Ty, V>),
}

impl<'a, Ty: ?Sized + Downcast, V: IntoBox<Ty>> Entry<'a, Ty, V> {
    /// Ensures a value is in the entry by inserting the
    /// default if empty, and returns
    /// a mutable reference to the value in the entry.
    #[inline]
    pub fn or_insert(self, default: V) -> &'a mut V {
        match self {
            Entry::Occupied(inner) => inner.into_mut(),
            Entry::Vacant(inner) => inner.insert(default),
        }
    }

    /// Ensures a value is in the entry by inserting the
    /// result of the default function if empty, and
    /// returns a mutable reference to the value in the
    /// entry.
    #[inline]
    pub fn or_insert_with<F: FnOnce() -> V>(self, default: F) -> &'a mut V {
        match self {
            Entry::Occupied(inner) => inner.into_mut(),
            Entry::Vacant(inner) => inner.insert(default()),
        }
    }

    /// Ensures a value is in the entry by inserting the
    /// default value if empty, and returns a mutable
    /// reference to the value in the entry.
    #[inline]
    pub fn or_default(self) -> &'a mut V
    where
        V: Default,
    {
        match self {
            Entry::Occupied(inner) => inner.into_mut(),
            Entry::Vacant(inner) => inner.insert(Default::default()),
        }
    }

    /// Provides in-place mutable access to an occupied
    /// entry before any potential inserts into the map.
    #[inline]
    pub fn and_modify<F: FnOnce(&mut V)>(self, f: F) -> Self {
        match self {
            Entry::Occupied(mut inner) => {
                f(inner.get_mut());
                Entry::Occupied(inner)
            }
            Entry::Vacant(inner) => Entry::Vacant(inner),
        }
    }

    // Additional stable methods (as of 1.60.0-nightly) that
    // could be added: insert_entry(self, value: V) ->
    // OccupiedEntry<'a, K, V>                     (1.59.0)
}

impl<'a, Ty: ?Sized + Downcast, V: IntoBox<Ty>> OccupiedEntry<'a, Ty, V> {
    /// Gets a reference to the value in the entry
    #[inline]
    pub fn get(&self) -> &V {
        unsafe { self.inner.get().downcast_ref_unchecked() }
    }

    /// Gets a mutable reference to the value in the entry
    #[inline]
    pub fn get_mut(&mut self) -> &mut V {
        unsafe { self.inner.get_mut().downcast_mut_unchecked() }
    }

    /// Converts the OccupiedEntry into a mutable reference
    /// to the value in the entry with a lifetime bound
    /// to the collection itself
    #[inline]
    pub fn into_mut(self) -> &'a mut V {
        unsafe { self.inner.into_mut().downcast_mut_unchecked() }
    }

    /// Sets the value of the entry, and returns the entry's
    /// old value
    #[inline]
    pub fn insert(&mut self, value: V) -> V {
        unsafe { *self.inner.insert(value.into_box()).downcast_unchecked() }
    }

    /// Takes the value out of the entry, and returns it
    #[inline]
    pub fn remove(self) -> V {
        unsafe { *self.inner.remove().downcast_unchecked() }
    }
}

impl<'a, Ty: ?Sized + Downcast, V: IntoBox<Ty>> VacantEntry<'a, Ty, V> {
    /// Sets the value of the entry with the VacantEntry's
    /// key, and returns a mutable reference to it
    #[inline]
    pub fn insert(self, value: V) -> &'a mut V {
        unsafe { self.inner.insert(value.into_box()).downcast_mut_unchecked() }
    }
}

/// The map key for [`TypeMap`].
///
/// Typically, this can be considered an implementation
/// detail of the library, though if you're not
/// using [`tmkey!`] for deriving keys it may be useful.
///
/// There are two variants of this type:
///
/// 1. A [`TypeId`], probably of the relevant [`Key`]
/// 2. A prehashed value stored as a `u64`
///
/// The second type allows for a limited form of runtime
/// dynamism, which the caller is responsible for ensuring
/// that the `u64` -> `T` pair is singular.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct TypeMapKey {
    k: TypeKey,
}

impl TypeMapKey {
    /// Create a new [`TypeId`] variant
    pub const fn with_typeid(id: TypeId) -> Self {
        Self { k: TypeKey::Id(id) }
    }

    /// Create a new externally hashed variant
    pub const fn with_exthash(value: u64) -> Self {
        Self {
            k: TypeKey::ExtHash(value),
        }
    }
}

impl From<TypeId> for TypeMapKey {
    fn from(id: TypeId) -> Self {
        Self::with_typeid(id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum TypeKey {
    /// A normal [`TypeId`]
    Id(TypeId),
    /// A value that has already been hashed, externally
    ExtHash(u64),
}

impl std::hash::Hash for TypeKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            TypeKey::Id(id) => id.hash(state),
            TypeKey::ExtHash(pre) => pre.hash(state),
        }
    }
}

/// A hasher designed to eke a little more speed out, given
/// `TypeId`’s known characteristics.
///
/// This hasher effectively acts as a noop Hash
/// implementation, as we implicitly trust Rust's `TypeId`
/// uniqueness guarantee.
#[derive(Default)]
pub struct TypeIdHasher {
    value: u64,
}

impl Hasher for TypeIdHasher {
    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        // This expects to receive exactly one 64-bit value, and
        // there’s no realistic chance of that changing, but
        // I don’t want to depend on something that isn’t expressly
        // part of the contract for safety. But I’m OK with
        // release builds putting everything in one bucket
        // if it *did* change (and debug builds panicking).
        debug_assert_eq!(bytes.len(), 8);
        let _ = bytes
            .try_into()
            .map(|array| self.value = u64::from_ne_bytes(array));
    }

    #[inline]
    fn finish(&self) -> u64 {
        self.value
    }
}

#[cfg(test)]
mod tests;
