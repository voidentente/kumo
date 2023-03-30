use bevy::prelude::*;
//use bevy_egui::{EguiContext, EguiContexts};

pub struct InterfacePlugin;

/*
#[derive(States, Clone, Debug, Hash, PartialEq, Eq)]
pub enum InterfaceState {
    Displaying
}

impl Default for InterfaceState {
    fn default() -> Self {
        Self::Displaying
    }
}
 */

impl Plugin for InterfacePlugin {
    fn build(&self, _app: &mut App) {
        /*
        app.add_plugin(bevy_egui::EguiPlugin);
        app.add_state::<InterfaceState>();
        app.add_systems(OnEnter(InterfaceState::Displaying), setup);
        app.add_systems(OnExit(InterfaceState::Displaying), cleanup);
        app.add_systems(OnUpdate(InterfaceState::Displaying), update);
        app.add_systems(Update, update);
         */
    }
}

/*
fn _setup() {}

fn _cleanup() {}

fn _update(mut ctx: EguiContexts) {
    egui::Window::new("Hello").show(ctx.ctx_mut(), |ui| {
        ui.label("world");
    });
}
 */