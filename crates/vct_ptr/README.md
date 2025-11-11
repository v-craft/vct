# V-Craft Pointer

> Inspired by [bevy_ptr](https://github.com/bevyengine/bevy/blob/main/crates/bevy_ptr/README.md).

This crate provides several thin wrappers around Rust standard pointers that are used frequently in the ECS modules.

A pointer that is safe to use should satisfy the following:

- The pointer is non-null (not a zero value).
- The pointer refers to an address within a valid region (stack or heap).
- The pointer points to an initialized instance of `T`.
- The pointer meets the alignment requirements.
- The lifetime associated with the pointer is valid for the pointed-to instance.
- Rust aliasing rules are respected: at any time there is either one mutable borrow or any number of immutable borrows.

## Standard Pointers

|Pointer Type       |Lifetime'ed|Mutable|Strongly Typed|Aligned|Not Null|Forbids Aliasing|Forbids Arithmetic|
|-------------------|-----------|-------|--------------|-------|--------|----------------|------------------|
|`Box<T>`           |Owned      |Yes    |Yes           |Yes    |Yes     |Yes             |Yes               |
|`&'a mut T`        |Yes        |Yes    |Yes           |Yes    |Yes     |Yes             |Yes               |
|`&'a T`            |Yes        |No     |Yes           |Yes    |Yes     |No              |Yes               |
|`&'a UnsafeCell<T>`|Yes        |Maybe  |Yes           |Yes    |Yes     |Yes             |Yes               |
|`NonNull<T>`       |No         |Yes    |Yes           |No     |Yes     |No              |No                |
|`*const T`         |No         |No     |Yes           |No     |No      |No              |No                |
|`*mut T`           |No         |Yes    |Yes           |No     |No      |No              |No                |
|`*const ()`        |No         |No     |No            |No     |No      |No              |No                |
|`*mut ()`          |No         |Yes    |No            |No     |No      |No              |No                |

## Available in this project

|Pointer Type         |Lifetime'ed|Mutable|Strongly Typed|Aligned|Not Null|Forbids Aliasing|Forbids Arithmetic|
|---------------------|-----------|-------|--------------|-------|--------|----------------|------------------|
|`ConstNonNull<T>`    |No         |No     |Yes           |No     |Yes     |No              |Yes               |
|`Ptr<'a>`            |Yes        |No     |No            |Maybe  |Yes     |No              |No                |
|`PtrMut<'a>`         |Yes        |Yes    |No            |Maybe  |Yes     |Yes             |No                |
|`ManualPtr<'a>`      |Yes        |Yes    |No            |Maybe  |Yes     |Yes             |No                |
|`AutoPtr<'a, T>`     |Yes        |Yes    |Yes           |Maybe  |Yes     |Yes             |Yes               |
|`ThinSlicePtr<'a, T>`|Yes        |No     |Yes           |Yes    |Yes     |Yes             |Yes               |

`ConstNonNull<T>` is similar to `NonNull<T>`, but it cannot be converted into a pointer that allows mutating the target.

`Ptr<'a>` and `PtrMut<'a>` are like `NonNull<()>` but carry a lifetime and support alignment checks to approximate the safety semantics of `&T` and `&mut T`.

`ManualPtr<'a>`—the name references `ManuallyDrop`—gives manual control over calling `Drop::drop`. Holding a `ManualPtr` without acting on it does not affect the pointed object; you can optionally consume it with a `make` function (to run/destruct the object) or call `drop_as` to invoke the target's `drop` early. (This corresponds to bevy_ptr's `OwningPtr`, though the name "Owning" can be misleading since it does not allocate or free memory.)

`ManualPtr<'a>` is untyped so it cannot automatically call `drop`. `AutoPtr<'a, T>` is a typed version of `ManualPtr<'a>` and will, by default, call the pointed object's `drop` when it itself is dropped. (This matches bevy_ptr's `MovingPtr`; the term "Moving" is used there because it’s often used to cheaply "move" large objects—`Auto` parallels `ManuallyDrop`.)

`ThinSlicePtr` is a slice type without length metadata. It is lighter-weight but cannot perform bounds checks; element access is unsafe. (In debug builds it retains length information for checks.)
