# msg_post_process

[![CI](https://github.com/MolecularSadism/msg_post_process/workflows/CI/badge.svg)](https://github.com/MolecularSadism/msg_post_process/actions)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](https://github.com/MolecularSadism/msg_post_process#license)
[![Bevy](https://img.shields.io/badge/Bevy-0.19-blue.svg)](https://bevyengine.org/)
[![Rust](https://img.shields.io/badge/rust-2024%20edition-orange.svg)](https://www.rust-lang.org/)

Thin ergonomics for ordering full-screen post-process passes in
[Bevy](https://bevyengine.org)'s camera render schedules.

A post-process pass is the standard Bevy pattern: a render system in a camera
schedule (`Core2d`, `Core3d`, or a custom one) that reads the current view's
`ViewTarget` through a `ViewQuery`, calls `post_process_write`, and draws a
full-screen triangle with a `RenderContext` (see Bevy's
`custom_post_processing` example). This crate adds nothing to that system — it
provides one extension trait, `PostProcessAppExt`, so a plugin can add its pass
under a `SystemSet` label and wire that label's ordering in one place:

```rust,ignore
use bevy::core_pipeline::{Core2d, Core2dSystems};
use msg_post_process::PostProcessAppExt;

#[derive(Debug, Clone, PartialEq, Eq, Hash, SystemSet)]
struct MyEffectSystems;

app.add_post_process_pass(Core2d, MyEffectSystems, my_effect_pass);
app.render_between(
    Core2d,
    MyEffectSystems,
    Core2dSystems::MainPass,
    Core2dSystems::PostProcess,
);
```

Every method takes the target schedule first — pass
`bevy::core_pipeline::Core3d` (and `Core3dSystems` anchors) to order a pass in
the 3D pipeline instead. Ordering targets are anything a system can be ordered
against: another pass's label, a core set (`Core2dSystems::EarlyPostProcess`
holds the HDR passes before tonemapping), or a render system function such as
`bevy::core_pipeline::tonemapping::tonemapping`.

Passes and ordering go into the render sub-app's camera schedule, which the
core pipeline plugin creates, so call these from a plugin that builds after it
— `Plugin::finish` is always late enough. Every method is a no-op when the app
has no `RenderApp` (e.g. headless tests), so callers don't need to guard for it.

### Migrating from 0.1 (Bevy 0.18)

Bevy 0.19 replaced the render graph with camera schedules. `ViewNode`s become
systems, `RenderLabel`s become `SystemSet` labels, and
`add_post_process_node::<Node>(graph, label)` becomes
`add_post_process_pass(schedule, label, system)`. The old `Node2d` anchors map
to sets: `EndMainPass → StartMainPassPostProcessing` is
`Core2dSystems::MainPass → Core2dSystems::PostProcess` (or simply
`Core2dSystems::EarlyPostProcess`), and `StartMainPass` is
`Core2dSystems::MainPass` as a `render_before` target.

## Installation

Not yet on crates.io — depend on the repository:

```toml
[dependencies]
msg_post_process = { git = "https://github.com/MolecularSadism/msg_post_process", tag = "v0.2.0" }
```

## Compatibility

| msg_post_process | Bevy | Rust |
|------------------|------|------|
| 0.2 | 0.19 | 1.95+ (edition 2024) |
| 0.1 | 0.18 | 1.85+ (edition 2024) |

## License

Dual-licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at
your option.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this work, as defined in the Apache-2.0 license, shall be
dual-licensed as above, without any additional terms or conditions.
