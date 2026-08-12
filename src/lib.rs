//! Thin ergonomics for ordering full-screen post-process passes in Bevy's
//! render graph.
//!
//! A post-process pass is the standard Bevy pattern: a
//! [`ViewNode`] that reads the camera's
//! [`ViewTarget`](bevy::render::view::ViewTarget) via `post_process_write` and
//! draws a full-screen triangle (see Bevy's `custom_post_processing` example).
//! This crate adds nothing to that node — it only provides an extension trait
//! so a plugin can add its node to a render sub-graph
//! ([`Core2d`](bevy::core_pipeline::core_2d::graph::Core2d),
//! [`Core3d`](bevy::core_pipeline::core_3d::graph::Core3d), or a custom one)
//! and wire its ordering edges in one place, reading like prose:
//!
//! ```
//! use bevy::prelude::*;
//! use bevy::core_pipeline::core_2d::graph::{Core2d, Node2d};
//! use bevy::render::render_graph::RenderLabel;
//! use msg_post_process::PostProcessAppExt;
//!
//! #[derive(Debug, Clone, PartialEq, Eq, Hash, RenderLabel)]
//! struct MyEffectLabel;
//!
//! let mut app = App::new();
//! app.add_plugins(MinimalPlugins);
//! // Place the pass in the post-main-pass band. (No render app under
//! // MinimalPlugins, so this is a no-op here — it shows the call shape.)
//! app.render_between(
//!     Core2d,
//!     MyEffectLabel,
//!     Node2d::EndMainPass,
//!     Node2d::StartMainPassPostProcessing,
//! );
//! ```
//!
//! Edges are added immediately against the live render graph, so the referenced
//! nodes must already exist. Wire an edge from a plugin that runs after both
//! nodes are added — typically in `Plugin::finish`, which runs once every
//! plugin's `build` has completed.

use bevy::app::App;
use bevy::ecs::world::FromWorld;
use bevy::render::RenderApp;
use bevy::render::render_graph::{
    RenderGraphExt, RenderLabel, RenderSubGraph, ViewNode, ViewNodeRunner,
};

/// Extension methods for adding and ordering post-process render nodes.
///
/// Every method takes the target sub-graph first (e.g.
/// [`Core2d`](bevy::core_pipeline::core_2d::graph::Core2d) or
/// [`Core3d`](bevy::core_pipeline::core_3d::graph::Core3d)) and is a no-op
/// when the app has no [`RenderApp`] (e.g. headless tests), so callers don't
/// need to guard for it.
pub trait PostProcessAppExt {
    /// Adds a [`ViewNode`] to the given render sub-graph (wrapped in a
    /// [`ViewNodeRunner`]). Order it with the `render_*` methods below.
    fn add_post_process_node<N>(
        &mut self,
        graph: impl RenderSubGraph,
        label: impl RenderLabel,
    ) -> &mut Self
    where
        N: ViewNode + FromWorld + Send + Sync + 'static;

    /// Orders `node` to run after `after` (adds the edge `after → node`).
    fn render_after(
        &mut self,
        graph: impl RenderSubGraph,
        node: impl RenderLabel,
        after: impl RenderLabel,
    ) -> &mut Self;

    /// Orders `node` to run before `before` (adds the edge `node → before`).
    fn render_before(
        &mut self,
        graph: impl RenderSubGraph,
        node: impl RenderLabel,
        before: impl RenderLabel,
    ) -> &mut Self;

    /// Orders `node` to run between `after` and `before` (adds the edges
    /// `after → node → before`).
    fn render_between(
        &mut self,
        graph: impl RenderSubGraph,
        node: impl RenderLabel,
        after: impl RenderLabel,
        before: impl RenderLabel,
    ) -> &mut Self;
}

impl PostProcessAppExt for App {
    fn add_post_process_node<N>(
        &mut self,
        graph: impl RenderSubGraph,
        label: impl RenderLabel,
    ) -> &mut Self
    where
        N: ViewNode + FromWorld + Send + Sync + 'static,
    {
        if let Some(render_app) = self.get_sub_app_mut(RenderApp) {
            render_app.add_render_graph_node::<ViewNodeRunner<N>>(graph, label);
        }
        self
    }

    fn render_after(
        &mut self,
        graph: impl RenderSubGraph,
        node: impl RenderLabel,
        after: impl RenderLabel,
    ) -> &mut Self {
        if let Some(render_app) = self.get_sub_app_mut(RenderApp) {
            render_app.add_render_graph_edge(graph, after, node);
        }
        self
    }

    fn render_before(
        &mut self,
        graph: impl RenderSubGraph,
        node: impl RenderLabel,
        before: impl RenderLabel,
    ) -> &mut Self {
        if let Some(render_app) = self.get_sub_app_mut(RenderApp) {
            render_app.add_render_graph_edge(graph, node, before);
        }
        self
    }

    fn render_between(
        &mut self,
        graph: impl RenderSubGraph,
        node: impl RenderLabel,
        after: impl RenderLabel,
        before: impl RenderLabel,
    ) -> &mut Self {
        let graph = graph.intern();
        let node = node.intern();
        if let Some(render_app) = self.get_sub_app_mut(RenderApp) {
            render_app.add_render_graph_edge(graph, after, node);
            render_app.add_render_graph_edge(graph, node, before);
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::core_pipeline::core_2d::graph::{Core2d, Node2d};
    use bevy::core_pipeline::core_3d::graph::{Core3d, Node3d};
    use bevy::render::render_graph::RenderLabel;

    #[derive(Debug, Clone, PartialEq, Eq, Hash, RenderLabel)]
    struct TestLabel;

    #[test]
    fn ordering_methods_are_noops_without_a_render_app() {
        let mut app = App::new();
        app.render_after(Core2d, TestLabel, Node2d::EndMainPass)
            .render_before(Core2d, TestLabel, Node2d::StartMainPassPostProcessing)
            .render_between(
                Core2d,
                TestLabel,
                Node2d::EndMainPass,
                Node2d::StartMainPassPostProcessing,
            )
            .render_between(Core3d, TestLabel, Node3d::EndMainPass, Node3d::Tonemapping);
    }
}
