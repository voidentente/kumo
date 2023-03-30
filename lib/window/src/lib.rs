use bevy::prelude::*;

/// Returns the window configuration.
pub fn window() -> Window {
    Window {
        present_mode: bevy::window::PresentMode::Fifo,
        mode: bevy::window::WindowMode::BorderlessFullscreen,
        title: "Kumo".to_string(),
        resizable: false,
        decorations: false,
        focused: true,
        transparent: true,
        ..Default::default()
    }
}

pub const WINDOW_ICON: &[u8] = include_bytes!("../assets/kumo.png");

pub fn icon() -> winit::window::Icon {
    let image = image::load_from_memory_with_format(
        WINDOW_ICON,
        image::ImageFormat::Png,
    ).unwrap();
    let (width, height) = image::GenericImageView::dimensions(&image);
    winit::window::Icon::from_rgba(
        image.into_rgba8().into_raw(), width, height).unwrap()
}

#[derive(Resource)]
pub struct WindowEntity {
    pub id: Entity,
    /// This is an explicit flag to more thoroughly check whether this window is new.
    /// It replaces .is_changed on this resource.
    pub needs_icon: bool,
}

pub struct WindowPlugin;

impl Plugin for WindowPlugin {
    fn build(&self, app: &mut App) {
        // Setup Bevy's Window plugin.
        // We do not exit automatically!
        // The same instance can be brought up again
        // by simply starting Kumo again. See main.rs.
        app.add_plugin(bevy::window::WindowPlugin {
            primary_window: None,
            exit_condition: bevy::window::ExitCondition::DontExit,
            close_when_requested: true,
        });

        // We start with our window spawned.
        let window = app.world.spawn(window())
            .insert(bevy::window::PrimaryWindow).id();

        // We save our primary window entity id for later use.
        app.insert_resource(WindowEntity { id: window, needs_icon: true });

        // Before adding winit, we configure it to be lighter on resources.
        app.insert_resource(bevy::winit::WinitSettings {
            focused_mode: bevy::winit::UpdateMode::ReactiveLowPower {
                max_wait: std::time::Duration::from_millis(100)
            },
            unfocused_mode: bevy::winit::UpdateMode::ReactiveLowPower {
                max_wait: std::time::Duration::from_millis(1000)
            },
            ..Default::default()
        });
        app.add_plugin(bevy::winit::WinitPlugin);

        // Finally, we can setup our rendering.
        app.add_plugin(bevy::render::RenderPlugin {
            wgpu_settings: bevy::render::settings::WgpuSettings {
                power_preference: bevy::render::settings::PowerPreference::LowPower,
                ..Default::default()
            }
        });
        app.add_plugin(ImagePlugin::default_nearest());
        app.add_plugin(bevy::render::pipelined_rendering::PipelinedRenderingPlugin);
        app.add_plugin(bevy::core_pipeline::CorePipelinePlugin);
        app.insert_resource(ClearColor(Color::rgba(0.05, 0.05, 0.05, 0.5)));
        app.world.spawn(Camera2dBundle::default());
    }
}