# V-Craft Cross-Platform Support

> 参考 [bevy_platform](https://github.com/bevyengine/bevy/blob/main/crates/bevy_platform/README.md).

Rust 标准库提供了三个层级：core、alloc、std。

core 是最基础的语言核心功能；
alloc 代表了内存分配功能，并提供了 `String` `Vec` 等容器的实现；
std 是完整的标准库，额外包含文件、线程等操心系统 API。

> [Rust Platform Support](https://doc.rust-lang.org/nightly/rustc/platform-support.html)

理想状态下，游戏引擎面向的所有平台都支持 `core`、
基本支持 `alloc`（只需显式提供内存分配器）。

Win、Linux、Mac、Android等主流平台都支持完整的 Rust 标准库，甚至 Web 应用也能基本支持。
但对于主机平台甚至是嵌入式平台，很可能没有官方提供的 `std` 支持，需要直接调用底层接口。

常见做法是定义一个抽象层（不涉及渲染），包含所需的操作系统接口，不支持 rust std 的平台需要提供此抽象层的实现。

这将是一个非常庞大的工程，而且仅考虑 ECS 模块（不考虑资产与渲染模块）时很难确定有多少 API 是明确需要的。

因此，本库将直接使用 Rust 标准库，暂时不考虑嵌入式等平台的支持性。
