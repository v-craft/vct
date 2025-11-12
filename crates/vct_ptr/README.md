# V-Craft Pointer

> 参考 [bevy_ptr](https://github.com/bevyengine/bevy/blob/main/crates/bevy_ptr/README.md).

本库在 Rust 标准指针的基础上实现了一些额外的指针封装，它们将在 ECS 模块中被频繁使用。

一个安全的指针应该满足以下条件：

- 非空（非零值）
- 指向有效区间（栈区或堆区）
- 指向的目标对象已初始化
- 满足对齐要求
- 携带的生命周期标识对目标有效
- 满足 Rust 别名规则：任何时刻仅有一个可变引用或任意个不可变引用

## 标准库指针

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

## 本库提供的额外指针

|Pointer Type         |Lifetime'ed|Mutable|Strongly Typed|Aligned|Not Null|Forbids Aliasing|Forbids Arithmetic|
|---------------------|-----------|-------|--------------|-------|--------|----------------|------------------|
|`ConstNonNull<T>`    |No         |No     |Yes           |No     |Yes     |No              |Yes               |
|`Ptr<'a>`            |Yes        |No     |No            |Maybe  |Yes     |No              |No                |
|`PtrMut<'a>`         |Yes        |Yes    |No            |Maybe  |Yes     |Yes             |No                |
|`OwningPtr<'a>`      |Yes        |Yes    |No            |Maybe  |Yes     |Yes             |No                |
|`MovingPtr<'a, T>`   |Yes        |Yes    |Yes           |Maybe  |Yes     |Yes             |Yes               |
|`ThinSlicePtr<'a, T>`|Yes        |No     |Yes           |Yes    |Yes     |Yes             |Yes               |

`ConstNonNull<T>` 类似于 `NonNull<T>`，它是非空的常量指针，无法直接转换成可修改目标的类型（如可变引用）。

`Ptr<'a>` 和 `PtrMut<'a>` 类似于类型擦除的 `&T` 和 `&mut T` 。与裸指针相比，它们添加了生命周期标识并支持指针对齐检查，以尽可能接近引用的安全性。

`OwningPtr<'a>` 字面意思是“所有权指针”，你可以通过此指针消耗指向的目标或执行它的 `Drop::drop` 函数，类似 `&'a mut ManuallyDrop<dyn Any>`。
需要注意，它并不管理目标的内存资源，因此通常用于指向栈区对象或 `Vec` 等容器管理的对象。
存在此指针对象时不应该通过其他方式操作目标，即遵守 Rust 的别名规则。

`MovingPtr<'a, T>` 则更进一步，它携带了类型信息，因此在指针自身消亡时会自动执行目标的 `drop` 函数。
将其取名为 `Moving` 是因为此指针通常用于廉价“移动”大对象（参考C++的移动语义）。

> 大对象通常是一个小内存对象指向一个大内存对象，小对象的 `drop` 函数负责释放大对象。
> 我们使用 `MovingPtr` 指向这个小对象并托管它 `drop` 函数，移动数据时只需拷贝这小块内存。

`ThinSlicePtr` 是一个薄切片指针，它的特点是不包含切片的长度信息（只含一个指针），因此更加轻量。
但缺点是通过它访问元素是不安全的，无法进行边界检查（Debug模式将保留长度信息以方便调试）。
