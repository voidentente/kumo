#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

//! ---== PORT USAGE ==---
//! 11210 is used to achieve single instancing, and inter-process communication.
//! 11211 is used for DeviantArt's OAuth2 process.
//! 11212 is used by the Meilisearch server.
//! Configurability for these ports is planned, but not a priority.
//! If any of these ports shouldn't work for you, feel free to change the source code!

use bevy::prelude::*;

fn main() {
    // The first thing we do is check for another instance.
    // For Kumo it is sensible to not have multiple instances running simultaneously.
    // Instead of using `single_instance` for this, Kumo uses a TCP socket.
    // So we first check if our socket is already occupied:
    if let Ok(mut stream) = std::net::TcpStream::connect("127.0.0.1:11210") {
        // An instance is already bound to our address.
        // We send a dummy packet and then exit.
        std::io::Write::write_all(&mut stream, &[1]).unwrap();
        return;
    }
    // No other instance is running - we bind to our address.
    let listener = std::net::TcpListener::bind("127.0.0.1:11210").unwrap();
    println!("Now listening on 127.0.0.1:11210");
    // We can save a thread and channel by using the listener in a nonblocking way.
    listener.set_nonblocking(true).unwrap();

    // Since our presence is now established, we can do other important stuff,
    // like setting up tracing and logging.
    let _file_worker_guard = logging::setup().unwrap();

    // Then, we build our Bevy app.
    // This is somewhat expensive, which is why we don't do it at the complete start.
    let mut app = kumo::build();

    // Our listener is added to the app as resource, so we can use it in systems.
    app.insert_resource(InstanceListener(listener));
    // And we add the system that reads the listener:
    app.add_systems(Update, instance_system);

    // Finally, we start our app.
    app.run();
}

#[derive(Resource)]
struct InstanceListener(std::net::TcpListener);

fn instance_system(
    mut commands: Commands,
    mut window: ResMut<window::WindowEntity>,
    listener: Res<InstanceListener>,
    windows: NonSend<bevy::winit::WinitWindows>
) {
    // Check if we can set the icon of our window.
    if window.needs_icon {
        window.needs_icon = false;
        let window_id = windows.entity_to_winit.get(&window.id).unwrap();
        let winit_window = windows.windows.get(window_id).unwrap();
        winit_window.set_window_icon(Some(window::icon()));
    }

    // It is perfectly fine to only accept a single connection per Bevy update.
    if let Ok((stream, _addr)) = listener.0.accept() {
        // We restrict the buffer window so we cannot overrun our tiny buffer.
        let mut handle = std::io::Read::take(stream , 1);
        let mut buf = [0u8; 1];
        std::io::Read::read(&mut handle, &mut buf).unwrap();
        // The buffer starts off as [0], however, if we received message from another
        // Kumo instance, it will be set to [1]. We can then act accordingly:
        if buf[0] == 1u8 {
            // In this case, we want to create a new window,
            // if we don't already have a window.
            if commands.get_entity(window.id).is_none() {
                // A primary window doesn't exist, so we create one.
                // Since we must work with commands instead of with the app,
                // this code is essentially duplicate of that in `window`.
                // TODO: Clean single-instancing window creation up.
                window.id = commands.spawn(window::window())
                    .insert(bevy::window::PrimaryWindow).id();
                window.needs_icon = true;
            }
        }
    }
}