#![expect(unexpected_cfgs)]

use std::borrow::Cow;

use bevy::{
    ecs::{
        archetype::ArchetypeComponentId,
        component::{
            ComponentId,
            Tick,
        },
        query::Access,
        schedule::InternedSystemSet,
        system::SystemParamValidationError,
        world::{
            DeferredWorld,
            unsafe_world_cell::UnsafeWorldCell,
        },
    },
    prelude::*,
};
use variadics_please::all_tuples;

use crate::prelude::*;

/// A [`Flow`] is a collection of chained systems where input is passed from
/// system to system, modified by each one.
pub struct Flow<F> {
    systems: Vec<FlowSystem<F>>,
    components: Access<ComponentId>,
    archetypes: Access<ArchetypeComponentId>,
    initialized: bool,
    name: Cow<'static, str>,
}

impl<F> Default for Flow<F> {
    fn default() -> Self {
        Self {
            systems: Vec::new(),
            components: Access::new(),
            archetypes: Access::new(),
            initialized: false,
            name: "Flow[]".into(),
        }
    }
}

impl<F> Flow<F>
where
    F: 'static,
{
    /// Create a new, empty [`Flow`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new [`Flow`] from a single boxed system
    pub fn from_boxed_system(system: FlowSystem<F>) -> Self {
        let mut flow = Self::new();
        flow.push(system);
        flow
    }

    fn update(&mut self) {
        self.initialized = false;

        self.name = format!(
            "Flow[{}]",
            self.systems
                .iter()
                .map(|s| s.name())
                .collect::<Vec<_>>()
                .join(",")
        )
        .into();
    }

    /// Add a boxed [`System`] to the [`Flow`]
    pub fn push(&mut self, system: FlowSystem<F>) {
        self.systems.push(system);
        self.update();
    }

    /// Extend the [`Flow`]'s systems with the contents of the iterator
    pub fn extend(&mut self, systems: impl IntoIterator<Item = FlowSystem<F>>) {
        self.systems.extend(systems);
        self.update();
    }

    /// Merge the other [`Flow`]'s systems into this [`Flow`]
    pub fn merge(&mut self, flow: Self) {
        self.extend(flow.systems);
    }

    /// Whether or not the [`Flow`] is read-only
    ///
    /// Returns `None` if the system has not been initialized yet
    pub fn is_readonly(&self) -> Option<bool> {
        self.initialized
            .then(|| !self.components.has_any_write() && !self.archetypes.has_any_write())
    }
}

impl<F> System for Flow<F>
where
    F: 'static,
{
    type In = In<F>;
    type Out = F;

    fn name(&self) -> Cow<'static, str> {
        self.name.clone()
    }

    fn component_access(&self) -> &Access<ComponentId> {
        &self.components
    }

    fn archetype_component_access(&self) -> &Access<ArchetypeComponentId> {
        &self.archetypes
    }

    fn is_send(&self) -> bool {
        self.systems.iter().all(|s| s.is_send())
    }

    fn is_exclusive(&self) -> bool {
        self.systems.iter().any(|s| s.is_exclusive())
    }

    fn has_deferred(&self) -> bool {
        self.systems.iter().any(|s| s.has_deferred())
    }

    unsafe fn run_unsafe(
        &mut self,
        input: SystemIn<'_, Self>,
        world: UnsafeWorldCell,
    ) -> Self::Out {
        // SAFETY: Delegate to each contained system
        unsafe {
            self.systems
                .iter_mut()
                .fold(input, |last, system| system.run_unsafe(last, world))
        }
    }

    fn apply_deferred(&mut self, world: &mut World) {
        self.systems
            .iter_mut()
            .for_each(|s| s.apply_deferred(world));
    }

    fn queue_deferred(&mut self, mut world: DeferredWorld) {
        self.systems
            .iter_mut()
            .for_each(|s| s.queue_deferred(world.reborrow()));
    }

    unsafe fn validate_param_unsafe(
        &mut self,
        world: UnsafeWorldCell,
    ) -> Result<(), SystemParamValidationError> {
        // SAFETY: Any validation errors are returned by `try_for_each`
        unsafe {
            self.systems
                .iter_mut()
                .try_for_each(|s| s.validate_param_unsafe(world))
        }
    }

    fn validate_param(&mut self, world: &World) -> Result<(), SystemParamValidationError> {
        self.systems
            .iter_mut()
            .try_for_each(|s| s.validate_param(world))
    }

    fn initialize(&mut self, world: &mut World) {
        self.systems.iter_mut().for_each(|s| {
            s.initialize(world);
            self.components.extend(s.component_access());
        });

        self.initialized = true;
    }

    fn update_archetype_component_access(&mut self, world: UnsafeWorldCell) {
        self.systems.iter_mut().for_each(|s| {
            s.update_archetype_component_access(world);
            self.archetypes.extend(s.archetype_component_access());
        });
    }

    fn check_change_tick(&mut self, change_tick: Tick) {
        self.systems
            .iter_mut()
            .for_each(|s| s.check_change_tick(change_tick));
    }

    fn default_system_sets(&self) -> Vec<InternedSystemSet> {
        self.systems
            .iter()
            .flat_map(|s| s.default_system_sets())
            .collect()
    }

    fn get_last_run(&self) -> Tick {
        self.systems
            .first()
            .map(|s| s.get_last_run())
            .unwrap_or_default()
    }

    fn set_last_run(&mut self, last_run: Tick) {
        self.systems
            .iter_mut()
            .for_each(|s| s.set_last_run(last_run));
    }
}

mod marker {
    pub struct IsSystem;
    pub struct IsTuple;
}

/// Types that can convert into [`Flow`].
///
/// This trait is implemented for “systems” (functions whose arguments all
/// implement `SystemParam`), or tuples thereof.
#[diagnostic::on_unimplemented(
    message = "`{Self}` does not describe a valid flow",
    label = "invalid flow",
    note = r#"
every system needs `In<{F}>` as the first parameter and `{F}` as the return type
read-only flows must have read-only systems (no Query<&mut T>, ResMut<T>)
"#
)]
pub trait IntoFlowSystems<F: 'static, Marker>: Sized {
    /// Convert into [`Flow`] system.
    fn into_flow_systems(self) -> Flow<F>;
}

impl<F, M, S> IntoFlowSystems<F, (marker::IsSystem, M)> for S
where
    F: 'static,
    S: IntoSystem<In<F>, F, M>,
{
    fn into_flow_systems(self) -> Flow<F> {
        Flow::<F>::from_boxed_system(Box::new(IntoSystem::into_system(self)))
    }
}

macro_rules! impl_into_flow_systems {
    ($(#[$meta:meta])* $(($S:ident, $M:ident)),*) => {
        $(#[$meta])*
        #[allow(non_snake_case)]
        #[allow(unused_parens)]
        #[allow(unused_variables)]
        #[allow(unused_mut)]
        impl<F, $($M,)* $($S),*> IntoFlowSystems<F, (marker::IsTuple, $($M),*)> for ($($S,)*)
        where
            F: 'static,
            $($S: IntoFlowSystems<F, $M>),*
        {
            fn into_flow_systems(self) -> Flow<F> {
                let ($($S,)*) = self;
                let mut flow = Flow::new();
                $(flow.merge($S.into_flow_systems());)*
                flow
            }
        }
    };
}

all_tuples!(
    #[doc(fake_variadic)]
    impl_into_flow_systems,
    0,
    15,
    S,
    M
);

#[cfg(test)]
mod test {
    use bevy::prelude::*;

    use crate::flows::systems::IntoFlowSystems;

    #[derive(Default)]
    struct Builder {
        entities: Vec<Entity>,
    }

    #[derive(Resource, Default)]
    struct Extracted(bool);

    fn extract_transforms(In(mut b): In<Builder>, q: Query<Entity, With<Transform>>) -> Builder {
        b.entities.extend(q.iter());
        b
    }

    fn extract_vis(In(mut b): In<Builder>, q: Query<Entity, With<Visibility>>) -> Builder {
        b.entities.extend(q.iter());
        b
    }

    fn do_commands(In(b): In<Builder>, mut c: Commands) -> Builder {
        c.spawn_empty();
        b
    }

    fn update_resource(In(b): In<Builder>, mut res: ResMut<Extracted>) -> Builder {
        res.0 = true;
        b
    }

    #[test]
    fn test_flow_into_systems() {
        let mut flow_read = (extract_transforms, extract_vis).into_flow_systems();
        let mut flow_cmds = (extract_transforms, extract_vis, do_commands).into_flow_systems();
        let mut flow_write = (extract_transforms, extract_vis, update_resource).into_flow_systems();

        let mut app = App::new();

        app.init_resource::<Extracted>();

        let world = app.world_mut();

        world.spawn(Transform::default());
        world.spawn(Transform::default());
        world.spawn(Transform::default());
        world.spawn((Transform::default(), Visibility::default()));

        flow_read.initialize(world);
        flow_cmds.initialize(world);
        flow_write.initialize(world);

        assert_eq!(flow_read.is_readonly(), Some(true));
        assert_eq!(flow_cmds.is_readonly(), Some(true));
        assert_eq!(flow_write.is_readonly(), Some(false));

        let out = flow_read.run(Builder::default(), world);

        assert_eq!(out.entities.len(), 5);
    }
}
