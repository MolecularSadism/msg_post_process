# msg_post_process

Thin ergonomics for ordering full-screen post-process passes in
[Bevy](https://bevyengine.org)'s render graph.

A post-process pass is the standard Bevy pattern: a `ViewNode` that reads the
camera's `ViewTarget` via `post_process_write` and draws a full-screen triangle
(see Bevy's `custom_post_processing` example). This crate adds nothing to that
node — it provides one extension trait, `PostProcessAppExt`, so a plugin can
add its node to a render sub-graph (`Core2d`, `Core3d`, or a custom one) and
wire its ordering edges in one place:

```rust,ignore
use bevy::core_pipeline::core_2d::graph::{Core2d, Node2d};
use msg_post_process::PostProcessAppExt;

app.add_post_process_node::<MyEffectNode>(Core2d, MyEffectLabel);
app.render_between(
    Core2d,
    MyEffectLabel,
    Node2d::EndMainPass,
    Node2d::StartMainPassPostProcessing,
);
```

Every method takes the target sub-graph first — pass
`bevy::core_pipeline::core_3d::graph::Core3d` (and `Node3d` anchors) to order
a pass in the 3D pipeline instead.

`render_after` / `render_before` / `render_between` add edges immediately
against the live render graph, so the referenced nodes must already exist —
wire edges from a plugin that runs after both nodes are added, typically in
`Plugin::finish`. Every method is a no-op when the app has no `RenderApp`
(e.g. headless tests), so callers don't need to guard for it.

## Compatibility

| msg_post_process | Bevy | Rust |
|------------------|------|------|
| 0.1 | 0.18 | 1.85+ (edition 2024) |

## License

Dual-licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at
your option.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this work, as defined in the Apache-2.0 license, shall be
dual-licensed as above, without any additional terms or conditions.
