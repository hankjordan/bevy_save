use bevy::prelude::*;

use crate::prelude::*;

/// Extension trait that adds flow-related methods to Bevy's [`App`].
pub trait AppFlowExt {
    /// Add flow systems
    fn add_flows<F: 'static, M>(
        &mut self,
        flow: impl FlowLabel,
        systems: impl IntoFlowSystems<F, M>,
    ) -> &mut Self;
}

impl AppFlowExt for App {
    fn add_flows<F: 'static, M>(
        &mut self,
        flow: impl FlowLabel,
        systems: impl IntoFlowSystems<F, M>,
    ) -> &mut Self {
        self.init_resource::<Flows<F>>();

        self.world_mut()
            .resource_mut::<Flows<F>>()
            .add_systems(flow, systems);

        self
    }
}
