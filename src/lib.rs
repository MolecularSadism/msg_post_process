//! Thin ergonomics for ordering full-screen post-process passes in Bevy's
//! camera render schedules.
//!
//! A post-process pass is the standard Bevy pattern: a render system in a
//! camera schedule ([`Core2d`](bevy::core_pipeline::Core2d),
//! [`Core3d`](bevy::core_pipeline::Core3d), or a custom one) that reads the
//! current view's [`ViewTarget`](bevy::render::view::ViewTarget) through a
//! [`ViewQuery`](bevy::render::renderer::ViewQuery), calls
//! `post_process_write`, and draws a full-screen triangle with a
//! [`RenderContext`](bevy::render::renderer::RenderContext) (see Bevy's
//! `custom_post_processing` example). This crate adds nothing to that system —
//! it only provides an extension trait so a plugin can add its pass under a
//! [`SystemSet`] label and wire that label's ordering in one place, reading
//! like prose:
//!
//! ```
//! use bevy::prelude::*;
//! use bevy::core_pipeline::{Core2d, Core2dSystems};
//! use msg_post_process::PostProcessAppExt;
//!
//! #[derive(Debug, Clone, PartialEq, Eq, Hash, SystemSet)]
//! struct MyEffectPass;
//!
//! fn my_effect_pass() {}
//!
//! let mut app = App::new();
//! app.add_plugins(MinimalPlugins);
//! // Place the pass in the post-main-pass band. (No render app under
//! // MinimalPlugins, so this is a no-op here — it shows the call shape.)
//! app.add_post_process_pass(Core2d, MyEffectPass, my_effect_pass)
//!     .render_between(
//!         Core2d,
//!         MyEffectPass,
//!         Core2dSystems::MainPass,
//!         Core2dSystems::PostProcess,
//!     );
//! ```
//!
//! Ordering targets are anything a system can be ordered against: another
//! pass's label, a core set such as
//! [`Core2dSystems::EarlyPostProcess`](bevy::core_pipeline::Core2dSystems),
//! or a render system function such as
//! [`tonemapping`](bevy::core_pipeline::tonemapping::tonemapping).
//!
//! Passes and ordering are added to the render sub-app's camera schedule, which
//! the core pipeline plugin creates. Call these methods from a plugin that
//! builds after it — `Plugin::finish` is always late enough.

use bevy::app::App;
use bevy::ecs::schedule::{IntoScheduleConfigs, IntoSystemSet, ScheduleLabel, SystemSet};
use bevy::ecs::system::ScheduleSystem;
use bevy::render::RenderApp;

/// Extension methods for adding and ordering post-process render passes.
///
/// Every method takes the target camera schedule first (e.g.
/// [`Core2d`](bevy::core_pipeline::Core2d) or
/// [`Core3d`](bevy::core_pipeline::Core3d)) and is a no-op when the app has no
/// [`RenderApp`] (e.g. headless tests), so callers don't need to guard for it.
pub trait PostProcessAppExt {
    /// Adds render pass system(s) to the given camera schedule, inside the
    /// pass's own `label` set. Order the label with the `render_*` methods
    /// below.
    fn add_post_process_pass<M>(
        &mut self,
        schedule: impl ScheduleLabel,
        label: impl SystemSet,
        pass: impl IntoScheduleConfigs<ScheduleSystem, M>,
    ) -> &mut Self;

    /// Orders the `node` set to run after `after`.
    fn render_after<M>(
        &mut self,
        schedule: impl ScheduleLabel,
        node: impl SystemSet,
        after: impl IntoSystemSet<M>,
    ) -> &mut Self;

    /// Orders the `node` set to run before `before`.
    fn render_before<M>(
        &mut self,
        schedule: impl ScheduleLabel,
        node: impl SystemSet,
        before: impl IntoSystemSet<M>,
    ) -> &mut Self;

    /// Orders the `node` set to run after `after` and before `before`.
    fn render_between<MA, MB>(
        &mut self,
        schedule: impl ScheduleLabel,
        node: impl SystemSet,
        after: impl IntoSystemSet<MA>,
        before: impl IntoSystemSet<MB>,
    ) -> &mut Self;
}

impl PostProcessAppExt for App {
    fn add_post_process_pass<M>(
        &mut self,
        schedule: impl ScheduleLabel,
        label: impl SystemSet,
        pass: impl IntoScheduleConfigs<ScheduleSystem, M>,
    ) -> &mut Self {
        if let Some(render_app) = self.get_sub_app_mut(RenderApp) {
            render_app.add_systems(schedule, pass.in_set(label));
        }
        self
    }

    fn render_after<M>(
        &mut self,
        schedule: impl ScheduleLabel,
        node: impl SystemSet,
        after: impl IntoSystemSet<M>,
    ) -> &mut Self {
        if let Some(render_app) = self.get_sub_app_mut(RenderApp) {
            render_app.configure_sets(schedule, node.after(after));
        }
        self
    }

    fn render_before<M>(
        &mut self,
        schedule: impl ScheduleLabel,
        node: impl SystemSet,
        before: impl IntoSystemSet<M>,
    ) -> &mut Self {
        if let Some(render_app) = self.get_sub_app_mut(RenderApp) {
            render_app.configure_sets(schedule, node.before(before));
        }
        self
    }

    fn render_between<MA, MB>(
        &mut self,
        schedule: impl ScheduleLabel,
        node: impl SystemSet,
        after: impl IntoSystemSet<MA>,
        before: impl IntoSystemSet<MB>,
    ) -> &mut Self {
        if let Some(render_app) = self.get_sub_app_mut(RenderApp) {
            render_app.configure_sets(schedule, node.after(after).before(before));
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::app::SubApp;
    use bevy::core_pipeline::{Core2d, Core2dSystems, Core3d, Core3dSystems};
    use bevy::ecs::prelude::*;

    #[derive(Debug, Clone, PartialEq, Eq, Hash, SystemSet)]
    struct FirstPass;

    #[derive(Debug, Clone, PartialEq, Eq, Hash, SystemSet)]
    struct SecondPass;

    #[derive(Resource, Default)]
    struct Order(Vec<&'static str>);

    fn first(mut order: ResMut<Order>) {
        order.0.push("first");
    }

    fn second(mut order: ResMut<Order>) {
        order.0.push("second");
    }

    fn main_pass(mut order: ResMut<Order>) {
        order.0.push("main");
    }

    #[test]
    fn methods_are_noops_without_a_render_app() {
        let mut app = App::new();
        app.add_post_process_pass(Core2d, FirstPass, first)
            .render_after(Core2d, FirstPass, Core2dSystems::MainPass)
            .render_before(Core2d, FirstPass, Core2dSystems::PostProcess)
            .render_between(
                Core2d,
                FirstPass,
                Core2dSystems::MainPass,
                Core2dSystems::PostProcess,
            )
            .render_between(
                Core3d,
                FirstPass,
                Core3dSystems::MainPass,
                Core3dSystems::PostProcess,
            );
    }

    #[test]
    fn passes_run_in_the_wired_order() {
        let mut app = App::new();
        let mut render_app = SubApp::new();
        render_app.add_schedule(Core2d::base_schedule());
        render_app.init_resource::<Order>();
        app.insert_sub_app(RenderApp, render_app);

        // Added in reverse order; only the wired ordering decides the run order.
        app.add_post_process_pass(Core2d, SecondPass, second)
            .add_post_process_pass(Core2d, FirstPass, first)
            .render_between(Core2d, FirstPass, main_pass, Core2dSystems::PostProcess)
            .render_after(Core2d, SecondPass, FirstPass);
        let render_app = app.sub_app_mut(RenderApp);
        render_app.add_systems(Core2d, main_pass.in_set(Core2dSystems::MainPass));

        render_app.world_mut().run_schedule(Core2d);
        assert_eq!(
            render_app.world().resource::<Order>().0,
            ["main", "first", "second"]
        );
    }
}
