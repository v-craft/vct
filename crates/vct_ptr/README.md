# V-Craft Pointer

> Reference: [bevy_ptr](https://github.com/bevyengine/bevy/blob/main/crates/bevy_ptr/README.md).

This crate implements several pointer wrappers on top of Rust’s standard pointer types
that will be used frequently in the ECS module.

A safe pointer should satisfy the following conditions:

- Non-null
- Points to a valid memory range (stack or heap)
- The pointee is initialized
- Meets alignment requirements
- Carries a lifetime that is valid for the pointee
- Satisfies Rust aliasing rules: at any time either one mutable reference or any number of immutable references

## Standard library pointers

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

## Extra pointers in this crate

|Pointer Type         |Lifetime'ed|Mutable|Strongly Typed|Aligned|Not Null|Forbids Aliasing|Forbids Arithmetic|
|---------------------|-----------|-------|--------------|-------|--------|----------------|------------------|
|`ConstNonNull<T>`    |No         |No     |Yes           |No     |Yes     |No              |Yes               |
|`Ptr<'a>`            |Yes        |No     |No            |Maybe  |Yes     |No              |No                |
|`PtrMut<'a>`         |Yes        |Yes    |No            |Maybe  |Yes     |Yes             |No                |
|`OwningPtr<'a>`      |Yes        |Yes    |No            |Maybe  |Yes     |Yes             |No                |
|`MovingPtr<'a, T>`   |Yes        |Yes    |Yes           |Maybe  |Yes     |Yes             |Yes               |
|`ThinSlicePtr<'a, T>`|Yes        |No     |Yes           |Yes    |Yes     |Yes             |Yes               |

`ConstNonNull<T>` is similar to `NonNull<T>`:
a non-null pointer that cannot be used to obtain mutable references directly.

`Ptr<'a>` and `PtrMut<'a>` are like type-erased `&T` and `&mut T`. 
Compared to raw pointers they add a lifetime and optional alignment checks to approach the safety of references.

`OwningPtr<'a>` is an “ownership pointer”: 
it can be used to consume the pointee or invoke its `Drop::drop` (similar to `&'a mut ManuallyDrop<dyn Any>`). 
It does **not** manage memory of the pointee(so typically points to stack values or objects managed by other containers).

`MovingPtr<'a, T>` carries static type information and will call the pointee’s `Drop` impl when the pointer is dropped. 
It is designed for cheap “moves” of larger logical objects by pointing at a small value whose Drop frees the larger resource.

Note: large logical objects are often represented by a small control object whose drop releases the big resource. `MovingPtr` points to that small control object so moving it is cheap.

`ThinSlicePtr` is a thin slice pointer that does not store length (only a pointer), making it lighter.
Access through it is unsafe because bounds checks are not available;
in debug builds it may retain length info to help debugging.
