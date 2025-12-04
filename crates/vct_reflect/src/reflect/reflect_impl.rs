use crate::{
    cell::NonGenericTypeInfoCell,
    info::{DynamicTypePath, DynamicTyped, OpaqueInfo, ReflectKind, TypeInfo, TypePath, Typed},
    ops::{
        ApplyError, ReflectCloneError, ReflectMut, ReflectOwned, ReflectRef, array_debug,
        enum_debug, list_debug, map_debug, set_debug, struct_debug, tuple_debug,
        tuple_struct_debug,
    },
};
use alloc::{
    borrow::{Cow, ToOwned},
    boxed::Box,
};
use core::{
    any::{Any, TypeId},
    fmt,
};

pub trait Reflect: DynamicTypePath + DynamicTyped + Send + Sync + Any {
    /// Casts this type to a fully-reflected value.
    ///
    /// # Normal Impl
    ///
    /// ```ignore
    /// #[inline]
    /// fn as_reflect(&self) -> &dyn Reflect {
    ///     self
    /// }
    /// ```
    fn as_reflect(&self) -> &dyn Reflect;

    /// Casts this type to a mutable, fully-reflected value.
    ///
    /// # Normal Impl
    ///
    /// ```ignore
    /// #[inline]
    /// fn as_reflect_mut(&mut self) -> &mut dyn Reflect {
    ///     self
    /// }
    /// ```
    fn as_reflect_mut(&mut self) -> &mut dyn Reflect;

    /// Casts this type to a boxed, fully-reflected value.
    ///
    /// # Normal Impl
    ///
    /// ```ignore
    /// #[inline]
    /// fn into_reflect(self: Box<Self>) -> Box<dyn Reflect> {
    ///     self
    /// }
    /// ```
    fn into_reflect(self: Box<Self>) -> Box<dyn Reflect>;

    /// Performs a type-checked assignment of a reflected value to this value.
    ///
    /// This is a fixed type set, efficient but the type cannot be incorrect.
    /// Loose assignment, please use [`Reflect::try_apply`].
    ///
    /// # Normal Impl
    ///
    /// ```ignore
    /// #[inline]
    /// fn set(&mut self, value: Box<dyn Reflect>) -> Result<(), Box<dyn Reflect>> {
    ///     *self = *value.take::<Self>()?;
    ///     Ok(())
    /// }
    /// ```
    fn set(&mut self, value: Box<dyn Reflect>) -> Result<(), Box<dyn Reflect>>;
    // Normal impl: See macro `impl_cast_reflect_fn`.

    /// Indicates whether or not this type is a _dynamic_ data type.
    ///
    /// Normally, All other types should return false,
    /// meaning there is no need to implement it.
    #[inline]
    fn is_dynamic(&self) -> bool {
        false
    }

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
    #[inline]
    fn represented_type_info(&self) -> Option<&'static TypeInfo> {
        Some(self.reflect_type_info())
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
    fn to_dynamic(&self) -> Box<dyn Reflect> {
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
            ReflectRef::Opaque(value) => value.reflect_clone().unwrap_or_else(|_|{
                panic!("`Reflect::to_dynamic` failed because `Opaque` type `{}` is not support `reflect_clone`.", value.reflect_type_info().type_path());
            }),
        }
    }

    /// Applies a reflected value to this value.
    ///
    /// This `apply` function will not delete its original content beforehand.
    /// If self is a dynamic date type, please clean self up in advance (or use an empty container).
    fn try_apply(&mut self, value: &dyn Reflect) -> Result<(), ApplyError>;

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
    fn apply(&mut self, value: &dyn Reflect) {
        Reflect::try_apply(self, value).unwrap();
    }

    /// Attempts to clone `Self` using reflection.
    ///
    /// Unlike [`to_dynamic`], which generally returns a dynamic representation of `Self`,
    /// this method attempts create a clone of `Self` directly, if possible.
    #[inline]
    fn reflect_clone(&self) -> Result<Box<dyn Reflect>, ReflectCloneError> {
        Err(ReflectCloneError::NotImplemented {
            type_path: Cow::Owned(self.reflect_type_path().to_owned()),
        })
    }

    /// Returns a "partial equality" comparison result.
    ///
    /// If the underlying type does not support equality testing, returns `None`.
    #[inline]
    fn reflect_partial_eq(&self, _other: &dyn Reflect) -> Option<bool> {
        // Only Inline for default implement
        None
    }

    /// Returns a hash of the value (which includes the type).
    ///
    /// If the underlying type does not support hashing, returns `None`.
    #[inline]
    fn reflect_hash(&self) -> Option<u64> {
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
    fn reflect_debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.reflect_ref() {
            ReflectRef::Struct(dyn_struct) => struct_debug(dyn_struct, f),
            ReflectRef::TupleStruct(dyn_tuple_struct) => tuple_struct_debug(dyn_tuple_struct, f),
            ReflectRef::Tuple(dyn_tuple) => tuple_debug(dyn_tuple, f),
            ReflectRef::List(dyn_list) => list_debug(dyn_list, f),
            ReflectRef::Array(dyn_array) => array_debug(dyn_array, f),
            ReflectRef::Map(dyn_map) => map_debug(dyn_map, f),
            ReflectRef::Set(dyn_set) => set_debug(dyn_set, f),
            ReflectRef::Enum(dyn_enum) => enum_debug(dyn_enum, f),
            ReflectRef::Opaque(_) => write!(f, "Opaque({})", self.reflect_type_path()),
        }
    }
}

impl dyn Reflect {
    /// Returns `true` if the underlying value is of type `T`.
    #[inline]
    pub fn is<T: Any>(&self) -> bool {
        // Any::Type_id(self)
        self.type_id() == TypeId::of::<T>()
    }

    /// Downcasts the value to type `T` by reference.
    ///
    /// If the underlying value is not of type `T`, returns `None`.
    #[inline]
    pub fn downcast_ref<T: Any>(&self) -> Option<&T> {
        <dyn Any>::downcast_ref(self)
    }

    /// Downcasts the value to type `T` by mutable reference.
    ///
    /// If the underlying value is not of type `T`, returns `None`.
    #[inline]
    pub fn downcast_mut<T: Any>(&mut self) -> Option<&mut T> {
        <dyn Any>::downcast_mut(self)
    }

    /// Downcasts the value to type `T`, consuming the trait object.
    ///
    /// If the underlying value is not of type `T`, returns `Err(self)`.
    #[inline]
    pub fn downcast<T: Any>(self: Box<dyn Reflect>) -> Result<Box<T>, Box<dyn Reflect>> {
        if self.is::<T>() {
            // TODO: Use downcast_uncheck to reduce once type check
            // `Any::downcast_uncheck` is unstable now.
            Ok(<Box<dyn Any>>::downcast(self).unwrap())
        } else {
            Err(self)
        }
    }

    /// Downcasts the value to type `T`, unboxing and consuming the trait object.
    ///
    /// If the underlying value is not of type `T`, returns `Err(self)`.
    #[inline]
    pub fn take<T: Any>(self: Box<dyn Reflect>) -> Result<T, Box<dyn Reflect>> {
        self.downcast::<T>().map(|value| *value)
    }
}

impl fmt::Debug for dyn Reflect {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // PartialReflect::debug(self, f)
        self.reflect_debug(f)
    }
}

impl TypePath for dyn Reflect {
    #[inline]
    fn type_path() -> &'static str {
        "dyn vct_reflect::Reflect"
    }
    #[inline]
    fn type_name() -> &'static str {
        "dyn Reflect"
    }
}

impl Typed for dyn Reflect {
    /// This is the [`TypeInfo`] of [`dyn Reflect`],
    /// not the [`TypeInfo`] of the underlying data!!!!
    ///
    /// Use [`DynamicTyped::reflect_type_info`] to get underlying [`TypeInfo`].
    ///
    /// [`dyn Reflect`]: crate::Reflect
    fn type_info() -> &'static TypeInfo {
        static CELL: NonGenericTypeInfoCell = NonGenericTypeInfoCell::new();
        CELL.get_or_init(|| TypeInfo::Opaque(OpaqueInfo::new::<Self>()))
    }
}

macro_rules! impl_cast_reflect_fn {
    () => {
        #[inline]
        fn as_reflect(&self) -> &dyn Reflect {
            self
        }

        #[inline]
        fn as_reflect_mut(&mut self) -> &mut dyn Reflect {
            self
        }

        #[inline]
        fn into_reflect(self: Box<Self>) -> Box<dyn Reflect> {
            self
        }

        fn set(&mut self, value: Box<dyn Reflect>) -> Result<(), Box<dyn Reflect>> {
            // Manually inline
            *self = value.take::<Self>()?;
            Ok(())
        }
    };
}

pub(crate) use impl_cast_reflect_fn;
