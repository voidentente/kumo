use bevy::prelude::*;

use std::net::TcpListener;

const AUTH_URL: &str = "https://www.deviantart.com/oauth2/authorize";

const TOKEN_URL: &str = "https://www.deviantart.com/oauth2/token";

const REDIRECT_URL: &str = "http://localhost:11211";

pub struct DeviantArtPlugin;

impl Plugin for DeviantArtPlugin {
    fn build(&self, app: &mut App) {
        // Read our API credentials from file, for now.
        let exe_path = std::env::current_exe().unwrap();
        let exe_dir = exe_path.parent().unwrap();
        let da_config = std::fs::read_to_string(exe_dir.join("deviantart.txt")).unwrap();
        let (client_id, client_secret) = da_config.split_once('\n').unwrap();

        let client_id = oauth2::ClientId::new(client_id.to_owned());
        let client_secret = oauth2::ClientSecret::new(client_secret.to_owned());
        let auth_url = oauth2::AuthUrl::new(AUTH_URL.to_owned()).unwrap();
        let token_url = oauth2::TokenUrl::new(TOKEN_URL.to_owned()).unwrap();
        let redirect_url = oauth2::RedirectUrl::new(REDIRECT_URL.to_owned()).unwrap();

        let client = oauth2::basic::BasicClient::new(
            client_id,
            Some(client_secret),
            auth_url,
            Some(token_url)
        ).set_redirect_uri(redirect_url);

        app.insert_resource(OAuth2Client(client));
        app.add_systems(Update, start_system);
    }
}

#[derive(Resource)]
struct OAuth2Client(oauth2::basic::BasicClient);

#[derive(Resource)]
struct OAuth2Listener(TcpListener);

#[derive(Resource)]
struct CsrfToken(oauth2::CsrfToken);

#[derive(Resource)]
struct TokenResponse(oauth2::basic::BasicTokenResponse);

fn start_system(
    mut commands: Commands,
    client: Res<OAuth2Client>,
    keys: Res<Input<KeyCode>>,
    listener: Option<Res<OAuth2Listener>>,
    csrf_token: Option<Res<CsrfToken>>,
) {
    // Start the auth thingy.

    if keys.just_pressed(KeyCode::Tab) && listener.is_none() {
        info!("Initializing authentication...");

        let (auth_url, csrf_token) = client.0
            .authorize_url(oauth2::CsrfToken::new_random)
            .add_scope(oauth2::Scope::new("browse".to_string()))
            .url();

        let listener = TcpListener::bind("127.0.0.1:11211").unwrap();
        listener.set_nonblocking(true).unwrap();

        commands.insert_resource(OAuth2Listener(listener));
        commands.insert_resource(CsrfToken(csrf_token));

        webbrowser::open(auth_url.as_str())
            .expect("To authenticate with DeviantArt, a browser is required");
    }

    // Do the auth thingy.

    let (
        Some(listener),
        Some(csrf_token)
    ) = (listener, csrf_token) else {
        return;
    };

    info!("Waiting for incoming stream...");

    let Ok((mut stream, _addr)) = listener.0.accept() else {
        return;
    };

    info!("Got stream.");

    let code;
    let state;

    {
        let mut reader = std::io::BufReader::new(&stream);

        let mut request_line = String::new();
        std::io::BufRead::read_line(&mut reader, &mut request_line).unwrap();

        let redirect_url = request_line.split_whitespace().nth(1).unwrap();
        let url = oauth2::url::Url::parse(&("http://localhost".to_string() + redirect_url))
            .unwrap();

        let code_pair = url
            .query_pairs()
            .find(|pair| {
                let (key, _) = pair;
                key == "code"
            })
            .unwrap();

        let (_, value) = code_pair;
        code = oauth2::AuthorizationCode::new(value.into_owned());

        let state_pair = url
            .query_pairs()
            .find(|pair| {
                let (key, _) = pair;
                key == "state"
            })
            .unwrap();

        let (_, value) = state_pair;
        state = oauth2::CsrfToken::new(value.into_owned());
    }

    let message = "<script>close()</script>";

    let response = format!(
        "HTTP/1.1 200 OK\r\ncontent-length: {}\r\n\r\n{}",
        message.len(),
        message
    );
    std::io::Write::write_all(&mut stream, response.as_bytes()).unwrap();

    println!(
        "DeviantArt returned the following code:\n{}\n",
        code.secret()
    );
    println!(
        "DeviantArt returned the following state:\n{} (expected `{}`)\n",
        state.secret(),
        csrf_token.0.secret()
    );

    // Exchange the code with a token.
    let token_res = client.0
        .exchange_code(code)
        .request(oauth2::reqwest::http_client);

    println!(
        "DeviantArt returned the following token:\n{:?}\n",
        token_res
    );

    if let Ok(token) = token_res {
        let scopes = if let Some(scopes_vec) = oauth2::TokenResponse::scopes(&token) {
            scopes_vec
                .iter()
                .flat_map(|comma_separated| comma_separated.split(','))
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        };
        println!("DeviantArt returned the following scopes:\n{:?}\n", scopes);

        commands.insert_resource(TokenResponse(token));
    }

    // Finally, clean up the auth thingy.

    commands.remove_resource::<OAuth2Listener>();
    commands.remove_resource::<CsrfToken>();
}