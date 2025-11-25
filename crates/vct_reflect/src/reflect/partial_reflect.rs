use crate::{
    Reflect,
    info::{DynamicTypePath, ReflectKind, TypeInfo, TypePath},
    ops::{
        ApplyError, ReflectCloneError, ReflectMut, ReflectOwned, ReflectRef, array_debug,
        enum_debug, list_debug, map_debug, set_debug, struct_debug, tuple_debug,
        tuple_struct_debug,
    },
};
use alloc::{borrow::Cow, boxed::Box, string::ToString};
use core::{
    any::{Any, TypeId},
    fmt,
};
pub trait PartialReflect: DynamicTypePath + Send + Sync + 'static {
    /// Returns the [`TypeInfo`] of the type **represented** by this value.
    ///
    /// For most types, this will simply return their own `TypeInfo`.
    /// However, for dynamic types, such as [`DynamicStruct`] or [`DynamicList`],
    /// this will return the type their own `target_type` .
    ///
    /// This method is great if you have an instance of a type or a `dyn Reflect`,
    /// and want to access its [`TypeInfo`]. However, if this method is to be called
    /// frequently, consider using `TypeRegistry::get_type_info` as it can be more
    /// performant for such use cases.
    ///
    /// [`DynamicStruct`]: crate::ops::DynamicStruct
    /// [`DynamicList`]: crate::ops::DynamicList
    fn get_target_type_info(&self) -> Option<&'static TypeInfo>;

    /// Casts this type to a reflected value.
    fn as_partial_reflect(&self) -> &dyn PartialReflect;
    // Normal impl: fn (&self)-> &dyn PartialReflect{ self }

    /// Casts this type to a reflected value.
    fn as_partial_reflect_mut(&mut self) -> &mut dyn PartialReflect;
    // Normal impl: fn (&mut self)-> &mut dyn PartialReflect{ self }

    /// Casts this type to a boxed, reflected value.
    fn into_partial_reflect(self: Box<Self>) -> Box<dyn PartialReflect>;
    // Normal impl: fn (self: Box<Self>)-> Box<dyn PartialReflect>{ self }

    /// Attempts to cast this type to a [`Reflect`] value.
    fn try_as_reflect(&self) -> Option<&dyn Reflect>;
    // Normal impl: fn (&self)-> Option<&dyn Reflect>{ Some(self) }

    /// Attempts to cast this type to a mutable [`Reflect`] value.
    fn try_as_reflect_mut(&mut self) -> Option<&mut dyn Reflect>;
    // Normal impl: fn (&mut self)-> Option<&mut dyn Reflect>{ Some(self) }

    ///  Attempts to cast this type to a boxed [`Reflect`] value.
    fn try_into_reflect(self: Box<Self>) -> Result<Box<dyn Reflect>, Box<dyn PartialReflect>>;
    // Normal impl: fn (&mut self)-> Option<&mut dyn Reflect>{ Ok(self) }

    /// Applies a reflected value to this value.
    ///
    /// This `apply` function will not delete its original content beforehand.
    /// If self is a dynamic date type, please clean self up in advance (or use an empty container).
    ///
    /// This method are as follows:
    /// - If `Self` is a [`Struct`], then the value of each named field of `value` is
    ///   applied to the corresponding named field of `self`. Fields which are
    ///   not present in both structs are ignored.
    /// - If `Self` is a [`TupleStruct`] or [`Tuple`], then the value of each
    ///   numbered field is applied to the corresponding numbered field of
    ///   `self.` Fields which are not present in both values are ignored.
    /// - If `Self` is an [`Enum`], then the variant of `self` is `updated` to match
    ///   the variant of `value`. The corresponding fields of that variant are
    ///   applied from `value` onto `self`. Fields which are not present in both
    ///   values are ignored.
    /// - If `Self` is a [`List`] or [`Array`], then each element of `value` is applied
    ///   to the corresponding element of `self`. Up to `self.len()` items are applied,
    ///   and excess elements in `value` are appended to `self`.
    /// - If `Self` is a [`Map`], then for each key in `value`, the associated
    ///   value is applied to the value associated with the same key in `self`.
    ///   Keys which are not present in `self` are inserted, and keys from `self` which are not present in `value` are removed.
    /// - If `Self` is a [`Set`], then each element of `value` is applied to the corresponding
    ///   element of `Self`. If an element of `value` does not exist in `Self` then it is
    ///   cloned and inserted. If an element from `self` is not present in `value` then it is removed.
    /// - If `Self` is none of these, then `value` is downcast to `Self`, cloned, and
    ///   assigned to `self`.
    ///
    /// [`Struct`]: crate::ops::Struct
    /// [`TupleStruct`]: crate::ops::TupleStruct
    /// [`Tuple`]: crate::ops::Tuple
    /// [`Enum`]: crate::ops::Enum
    /// [`List`]: crate::ops::List
    /// [`Array`]: crate::ops::Array
    /// [`Map`]: crate::ops::Map
    /// [`Set`]: crate::ops::Set
    fn try_apply(&mut self, value: &dyn PartialReflect) -> Result<(), ApplyError>;

    /// Applies a reflected value to this value.
    ///
    /// Usually not required to implement, default to using try_apply.
    ///
    /// # Panics
    ///
    /// Derived implementations of this method will panic:
    /// - If the type of `value` is not of the same kind as `Self` (e.g. if `Self` is
    ///   a `List`, while `value` is a `Struct`).
    /// - If `Self` is any complex type and the corresponding fields or elements of
    ///   `self` and `value` are not of the same type.
    /// - If `Self` is an opaque type and `value` cannot be downcast to `Self`
    #[inline]
    fn apply(&mut self, value: &dyn PartialReflect) {
        PartialReflect::try_apply(self, value).unwrap();
    }

    /// Returns a zero-sized enumeration of "kinds" of type.
    fn reflect_kind(&self) -> ReflectKind;
    // Normal impl: fn (&self)-> ReflectKind{ ReflectKind::??? }

    /// Returns an immutable enumeration of "kinds" of type.
    fn reflect_ref(&self) -> ReflectRef<'_>;
    // Normal impl: fn (&self)-> ReflectRef{ ReflectRef::???::(self) }

    /// Returns a mutable enumeration of "kinds" of type.
    fn reflect_mut(&mut self) -> ReflectMut<'_>;
    // Normal impl: fn (&mut self)-> ReflectMut{ ReflectMut::???::(self) }

    /// Returns an owned enumeration of "kinds" of type.
    fn reflect_owned(self: Box<Self>) -> ReflectOwned;
    // Normal impl: fn (self: Box<Self>)-> ReflectOwned{ ReflectOwned::???::(self) }

    /// Indicates whether or not this type is a _dynamic_ data type.
    ///
    /// Normally, All other types should return false,
    /// meaning there is no need to implement it.
    #[inline]
    fn is_dynamic(&self) -> bool {
        false
    }

    /// Converts this reflected value into its dynamic representation based on its [kind].
    ///
    /// This function will clone a piece of data and will not be modified on its own.
    ///
    /// The new data will be completely dynamically typed (except for the bottom layer 'opaque'),
    /// rather than just modifying one layer.
    ///
    /// # Panics
    ///
    /// This method will panic if the [kind] is [opaque] and the call to [`reflect_clone`] fails.
    ///
    /// [kind]: PartialReflect::reflect_kind
    fn to_dynamic(&self) -> Box<dyn PartialReflect> {
        // Not inline: inline for dynamic objects is useless.
        match self.reflect_ref() {
            ReflectRef::Struct(dyn_struct) => Box::new(dyn_struct.to_dynamic_struct()),
            ReflectRef::TupleStruct(dyn_tuple_struct) => {
                Box::new(dyn_tuple_struct.to_dynamic_tuple_struct())
            }
            ReflectRef::Tuple(dyn_tuple) => Box::new(dyn_tuple.to_dynamic_tuple()),
            ReflectRef::List(dyn_list) => Box::new(dyn_list.to_dynamic_list()),
            ReflectRef::Array(dyn_array) => Box::new(dyn_array.to_dynamic_array()),
            ReflectRef::Map(dyn_map) => Box::new(dyn_map.to_dynamic_map()),
            ReflectRef::Set(dyn_set) => Box::new(dyn_set.to_dynamic_set()),
            ReflectRef::Enum(dyn_enum) => Box::new(dyn_enum.to_dynamic_enum()),
            ReflectRef::Opaque(value) => value.reflect_clone().unwrap().into_partial_reflect(),
        }
    }

    /// Attempts to clone `Self` using reflection.
    ///
    /// Unlike [`to_dynamic`], which generally returns a dynamic representation of `Self`,
    /// this method attempts create a clone of `Self` directly, if possible.
    fn reflect_clone(&self) -> Result<Box<dyn Reflect>, ReflectCloneError> {
        Err(ReflectCloneError::NotImplemented {
            type_path: Cow::Owned(self.reflect_type_path().to_string()),
        })
    }

    /// For a type implementing [`PartialReflect`], combines `reflect_clone` and
    /// `take` in a useful fashion, automatically constructing an appropriate
    /// [`ReflectCloneError`] if the downcast fails.
    ///
    /// In theory, the underlying type of the object generated by `reflect_clone`
    /// must be consistent with itself, so the type of take is itself.
    /// The required type is known, therefore it cannot be used for dyn Reflect objects.
    ///
    /// If the type supports [`Clone`] impl and the effect is the same as this function,
    /// it is recommended to use [`Clone`] directly for optimal performance.
    fn reflect_clone_take(&self) -> Result<Self, ReflectCloneError>
    where
        Self: TypePath + Any + Sized,
    {
        self.reflect_clone()?
            .take()
            .map_err(|_| ReflectCloneError::FailedDowncast {
                expected: Cow::Borrowed(<Self as TypePath>::type_path()),
                received: Cow::Owned(self.reflect_type_path().to_string()),
            })
    }

    /// Returns a hash of the value (which includes the type).
    ///
    /// If the underlying type does not support hashing, returns `None`.
    #[inline]
    fn reflect_hash(&self) -> Option<u64> {
        None
    }

    /// Returns a "partial equality" comparison result.
    ///
    /// If the underlying type does not support equality testing, returns `None`.
    #[inline]
    fn reflect_partial_eq(&self, _other: &dyn PartialReflect) -> Option<bool> {
        // Only Inline for default implement
        None
    }

    /// Debug formatter for the value.
    ///
    /// Any value that is not an implementor of other `Reflect` subtraits
    /// (e.g. [`List`], [`Map`]), will default to the format: `"Reflect(type_path)"`,
    /// where `type_path` is the [type path] of the underlying type.
    ///
    /// [`List`]: crate::List
    /// [`Map`]: crate::Map
    /// [type path]: TypePath::type_path
    fn debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.reflect_ref() {
            ReflectRef::Struct(dyn_struct) => struct_debug(dyn_struct, f),
            ReflectRef::TupleStruct(dyn_tuple_struct) => tuple_struct_debug(dyn_tuple_struct, f),
            ReflectRef::Tuple(dyn_tuple) => tuple_debug(dyn_tuple, f),
            ReflectRef::List(dyn_list) => list_debug(dyn_list, f),
            ReflectRef::Array(dyn_array) => array_debug(dyn_array, f),
            ReflectRef::Map(dyn_map) => map_debug(dyn_map, f),
            ReflectRef::Set(dyn_set) => set_debug(dyn_set, f),
            ReflectRef::Enum(dyn_enum) => enum_debug(dyn_enum, f),
            ReflectRef::Opaque(_) => write!(f, "Reflect({})", self.reflect_type_path()),
        }
    }
}

impl dyn PartialReflect {
    /// Returns `true` if the underlying value represents the type `T`
    #[inline]
    pub fn target_is<T: Reflect>(&self) -> bool {
        self.get_target_type_info()
            .is_some_and(|info| info.type_id() == TypeId::of::<T>())
    }

    /// Try Downcasts the value to type `T`, consuming the trait object.
    #[inline]
    pub fn try_downcast<T: Any>(
        self: Box<dyn PartialReflect>,
    ) -> Result<Box<T>, Box<dyn PartialReflect>> {
        // self.try_into_reflect()?.downcast::<T>()
        // .map_err(|err| err as Box<dyn PartialReflect>)
        // ↓ Manual inline
        match self.try_into_reflect()?.downcast::<T>() {
            Ok(val) => Ok(val),
            Err(err) => Err(err),
        }
    }

    /// Try Downcasts the value to type `T`, unboxing and consuming the trait object.
    #[inline]
    pub fn try_take<T: Any>(self: Box<dyn PartialReflect>) -> Result<T, Box<dyn PartialReflect>> {
        // self.try_into_reflect()?.take::<T>()
        // .map_err(|err| err as Box<dyn PartialReflect>)
        // ↓ Manual inline, merge two `map()`
        match self.try_into_reflect()?.downcast::<T>() {
            Ok(val) => Ok(*val),
            Err(err) => Err(err),
        }
    }

    /// Try Downcasts the value to type `T` by reference.
    #[inline]
    pub fn try_downcast_ref<T: Any>(&self) -> Option<&T> {
        self.try_as_reflect()?.downcast_ref()
    }

    /// Try Downcasts the value to type `T` by mutable reference.
    #[inline]
    pub fn try_downcast_mut<T: Any>(&mut self) -> Option<&mut T> {
        self.try_as_reflect_mut()?.downcast_mut()
    }
}

impl fmt::Debug for dyn PartialReflect {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.debug(f)
    }
}

impl TypePath for dyn PartialReflect {
    #[inline]
    fn type_path() -> &'static str {
        "dyn vct_reflect::PartialReflect"
    }

    #[inline]
    fn short_type_path() -> &'static str {
        "dyn PartialReflect"
    }
}
