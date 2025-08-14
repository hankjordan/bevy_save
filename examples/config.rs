use bevy::prelude::*;
use bevy_egui::{
    EguiContexts,
    EguiPlugin,
    EguiPrimaryContextPass,
    egui,
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_save::prelude::*;
use serde::{
    Deserialize,
    Serialize,
};

#[derive(Resource, Clone, Copy, Serialize, Deserialize)]
struct ControlsConfig {
    forwards: KeyCode,
    backwards: KeyCode,
    jump: KeyCode,
}

impl Default for ControlsConfig {
    fn default() -> Self {
        Self {
            forwards: KeyCode::KeyW,
            backwards: KeyCode::KeyS,
            jump: KeyCode::Space,
        }
    }
}

#[derive(Resource, Clone, Copy, Serialize, Deserialize)]
struct DisplayConfig {
    screen_x: f32,
    screen_y: f32,
    scale: f32,
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            screen_x: 1920.0,
            screen_y: 1080.0,
            scale: 1.0,
        }
    }
}

// Without using something like Vec<Box<dyn PartialReflect>>,
// we have to name each captured type explicitly.
#[derive(Default, Serialize, Deserialize)]
struct ConfigCapture {
    controls: Option<ControlsConfig>,
    display: Option<DisplayConfig>,
}

#[derive(Clone, Copy)]
struct ConfigPathway;

impl Pathway for ConfigPathway {
    type Capture = ConfigCapture;
    type Backend = DefaultDebugBackend;
    type Format = DefaultDebugFormat;

    type Key<'a> = &'a str;

    fn key(&self) -> Self::Key<'_> {
        "examples/saves/config"
    }

    fn capture(&self, _world: &World) -> impl bevy_save::prelude::FlowLabel {
        ConfigCaptureFlow
    }

    fn apply(&self, _world: &World) -> impl bevy_save::prelude::FlowLabel {
        ConfigApplyFlow
    }
}

#[derive(Hash, Debug, PartialEq, Eq, Clone, Copy, FlowLabel)]
pub struct ConfigCaptureFlow;

#[derive(Hash, Debug, PartialEq, Eq, Clone, Copy, FlowLabel)]
pub struct ConfigApplyFlow;

fn capture_resource<B: 'static, R: Resource>(
    capture: impl Fn(&mut B, &R) + Send + Sync + 'static,
) -> impl System<In = In<B>, Out = B> {
    IntoSystem::into_system(move |In(mut cap): In<B>, res: Res<R>| -> B {
        capture(&mut cap, &*res);
        cap
    })
}

fn apply_resource<B: 'static, R: Resource>(
    apply: impl Fn(&mut B) -> R + Send + Sync + 'static,
) -> impl System<In = In<B>, Out = B> {
    IntoSystem::into_system(move |In(mut cap): In<B>, mut cmds: Commands| -> B {
        cmds.insert_resource(apply(&mut cap));
        cap
    })
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

#[derive(Resource, Default)]
struct TriggerState {
    reset: bool,
    save: bool,
    load: bool,
}

fn handle_save_input(world: &mut World) {
    let mut trigger = world.resource_mut::<TriggerState>();

    if trigger.reset {
        info!("Resetting config");
        trigger.reset = false;
        *world.resource_mut::<ControlsConfig>() = Default::default();
        *world.resource_mut::<DisplayConfig>() = Default::default();
    }

    let mut trigger = world.resource_mut::<TriggerState>();

    if trigger.save {
        info!("Saving config");
        trigger.save = false;
        world.save(&ConfigPathway).expect("Failed to save");
    }

    let mut trigger = world.resource_mut::<TriggerState>();

    if trigger.load {
        info!("Loading config");
        trigger.load = false;
        world.load(&ConfigPathway).expect("Failed to load");
    }
}

fn render_controls(
    mut trigger: ResMut<TriggerState>,
    mut controls: ResMut<ControlsConfig>,
    mut display: ResMut<DisplayConfig>,
    mut ctxs: EguiContexts,
) {
    let ctx = ctxs.ctx_mut().expect("Failed to initialize egui");

    egui::Window::new("Config").show(ctx, |ui| {
        ui.heading("Controls");

        egui::ComboBox::new("ctrls-f", "Forwards")
            .selected_text(format!("{:?}", controls.forwards))
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut controls.forwards, KeyCode::KeyW, "W");
                ui.selectable_value(&mut controls.forwards, KeyCode::ArrowUp, "Up");
                ui.selectable_value(&mut controls.forwards, KeyCode::KeyI, "I");
            });

        egui::ComboBox::new("ctrls-b", "Backwards")
            .selected_text(format!("{:?}", controls.backwards))
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut controls.backwards, KeyCode::KeyS, "S");
                ui.selectable_value(&mut controls.backwards, KeyCode::ArrowDown, "Down");
                ui.selectable_value(&mut controls.backwards, KeyCode::KeyK, "K");
            });

        egui::ComboBox::new("ctrls-j", "Jump")
            .selected_text(format!("{:?}", controls.jump))
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut controls.jump, KeyCode::Space, "Space");
                ui.selectable_value(&mut controls.jump, KeyCode::ShiftLeft, "Left Shift");
                ui.selectable_value(&mut controls.jump, KeyCode::ShiftRight, "Right Shift");
            });

        ui.heading("Display");

        ui.add(egui::Slider::new(&mut display.screen_x, 800.0..=3840.0).text("Resolution (X)"));
        ui.add(egui::Slider::new(&mut display.screen_y, 600.0..=2160.0).text("Resolution (Y)"));
        ui.add(egui::Slider::new(&mut display.scale, 0.25..=2.0).text("Scale"));

        ui.horizontal(|ui| {
            if ui.button("Reset").clicked() {
                trigger.reset = true;
            }

            if ui.button("Save").clicked() {
                trigger.save = true;
            }

            if ui.button("Load").clicked() {
                trigger.load = true;
            }
        });
    });
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.build().set(AssetPlugin {
                file_path: "examples/assets".to_owned(),
                ..default()
            }),
            // egui
            EguiPlugin::default(),
            // Our resources won't be displayed in the inspector because we're not using reflection
            WorldInspectorPlugin::new(),
            // Bevy Save
            SavePlugins,
        ))
        // Resources
        .init_resource::<TriggerState>()
        .init_resource::<ControlsConfig>()
        .init_resource::<DisplayConfig>()
        // Pathway
        .init_pathway::<ConfigPathway>()
        // Flows
        .add_flows(
            ConfigCaptureFlow,
            (
                capture_resource::<ConfigCapture, _>(|c, r| c.controls = Some(*r)),
                capture_resource::<ConfigCapture, _>(|c, r| c.display = Some(*r)),
            ),
        )
        .add_flows(
            ConfigApplyFlow,
            (
                apply_resource::<ConfigCapture, _>(|c| c.controls.unwrap_or_default()),
                apply_resource::<ConfigCapture, _>(|c| c.display.unwrap_or_default()),
            ),
        )
        .add_systems(Startup, setup_camera)
        .add_systems(Update, handle_save_input)
        .add_systems(EguiPrimaryContextPass, render_controls)
        .run();
}
