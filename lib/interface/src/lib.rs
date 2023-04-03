use bevy::prelude::*;

pub struct InterfacePlugin;

#[derive(States, Clone, Debug, Hash, PartialEq, Eq)]
pub enum InterfaceState {
    Displaying
}

impl Default for InterfaceState {
    fn default() -> Self {
        Self::Displaying
    }
}

impl Plugin for InterfacePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(bevy_egui::EguiPlugin);
        app.add_state::<InterfaceState>();

        app.add_system(setup.in_schedule(OnEnter(InterfaceState::Displaying)));
        app.add_system(cleanup.in_schedule(OnExit(InterfaceState::Displaying)));
        app.add_system(draw.in_set(OnUpdate(InterfaceState::Displaying)));
    }
}

fn setup() {}

fn cleanup() {}

fn draw(mut ctx: bevy_egui::EguiContexts, window: Res<window::WindowEntity>) {
    let Some(ctx) = ctx.try_ctx_for_window_mut(window.id) else {
        return;
    };

    egui::Area::new("taskbar").anchor(egui::Align2::CENTER_BOTTOM, egui::Vec2::default()).show(ctx, |ui| {
        egui::Frame::none().fill(egui::Color32::WHITE).show(ui, |ui| {
            ui.label("Hello World~");
        });
    });
}
