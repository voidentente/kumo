use bevy::prelude::*;

pub fn build() -> App {
    let mut app = App::new();

    app.add_plugin(TaskPoolPlugin::default());
    app.add_plugin(TypeRegistrationPlugin::default());
    app.add_plugin(FrameCountPlugin::default());
    app.add_plugin(bevy::diagnostic::DiagnosticsPlugin);
    app.add_plugin(bevy::a11y::AccessibilityPlugin);
    app.add_plugin(bevy::time::TimePlugin);
    app.add_plugin(bevy::input::InputPlugin);
    app.add_plugin(TransformPlugin);

    app.add_plugin(AssetPlugin {
        watch_for_changes: true,
        asset_folder: std::env::args()
            .find(|s| s.starts_with("--assets="))
            .map(|s| s.split_once('=').unwrap().1.to_owned())
            .unwrap_or(Default::default()),
    });

    app.add_plugin(window::WindowPlugin);

    app.add_plugin(meiliguard::MeilisearchPlugin);

    app.add_plugin(interface::InterfacePlugin);

    app.add_plugin(deviantart::DeviantArtPlugin);

    app
}
