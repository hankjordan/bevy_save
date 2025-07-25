#![expect(clippy::needless_pass_by_value)]

use std::collections::HashMap;

use bevy::prelude::*;

use crate::{
    flows::{
        FlowLabel,
        InternedFlowLabel,
    },
    prelude::*,
};

/// Resource that stores labels mapped to flows.
#[derive(Resource)]
pub struct Flows<F> {
    map: HashMap<InternedFlowLabel, Flow<F>>,
}

impl<F> Default for Flows<F> {
    fn default() -> Self {
        Self {
            map: HashMap::new(),
        }
    }
}

impl<F> Flows<F>
where
    F: 'static,
{
    /// Insert a new [`Flow`].
    pub fn insert_flow(&mut self, label: impl FlowLabel, flow: Flow<F>) {
        self.map.insert(label.intern(), flow);
    }

    /// Remove a [`Flow`].
    pub fn take_flow(&mut self, label: impl FlowLabel) -> Option<Flow<F>> {
        self.map.remove(&label.intern())
    }

    /// Add systems to the [`Flow`].
    pub fn add_systems<M>(&mut self, label: impl FlowLabel, systems: impl IntoFlowSystems<F, M>) {
        self.map
            .entry(label.intern())
            .or_default()
            .merge(systems.into_flow_systems());
    }
}

impl<F> Flows<F>
where
    Flow<F>: System,
{
    /// Initializes registered flows.
    pub fn initialize(&mut self, world: &mut World) {
        for flow in self.map.values_mut() {
            flow.initialize(world);
        }
    }
}

#[cfg(test)]
mod test {
    use bevy::prelude::*;

    use super::*;
    use crate as bevy_save;
    use crate::prelude::*;

    #[derive(Default)]
    struct Builder {
        entities: Vec<Entity>,
    }

    #[derive(Hash, Debug, PartialEq, Eq, Clone, Copy, FlowLabel)]
    struct ExampleFlow;

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

    #[test]
    fn test_flow_registry() {
        let mut flows = Flows::default();
        flows.add_systems(ExampleFlow, (extract_transforms, extract_vis, do_commands));
    }

    #[test]
    fn test_flow_app_ext() {
        App::new().add_flows(ExampleFlow, (extract_transforms, extract_vis, do_commands));
    }
}
