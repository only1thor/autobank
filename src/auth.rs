use std::collections::HashMap;
use std::sync::mpsc;
use tiny_http::{Response, Server};
use url::form_urlencoded;
use urlencoding::encode;

use crate::config::read_access_token;

pub fn auth(client_id: String) {
    // 1: Check if access token present and valid
    let access_token = read_access_token();

    if test_token(access_token) {
        println!("Access token valid");
        return;
    }

    println!("Access token not valid");
    // 2: If not: Check if refresh token present and valid. Refresh to get access token.

    // 3: If not: Start auth flow with get code, then get access token.

    let code = get_code(client_id);
    let access_token = get_access_token(code);
}

fn get_code(client_id: String) -> String {
    let port = 8321;
    let redirect_uri = format!("http://localhost:{port}");

    let server = Server::http(format!("127.0.0.1:{port}")).unwrap();

    // Channel to send the code from the server thread
    let (tx, rx) = mpsc::channel();

    // Spawn server thread
    std::thread::spawn(move || {
        for request in server.incoming_requests() {
            let query = request.url().split('?').nth(1).unwrap_or("");
            let params: HashMap<_, _> = form_urlencoded::parse(query.as_bytes())
                .into_owned()
                .collect();

            if let Some(code) = params.get("code").cloned() {
                let response =
                    Response::from_string("✅ Authentication complete! You can close this tab.");
                request.respond(response).unwrap();

                // Send code to main thread
                tx.send(code).unwrap();
                break; // exit server loop
            }
        }
    });

    // Open browser
    let auth_url = format!(
        "https://api.sparebank1.no/oauth/authorize?client_id={}&state=123&redirect_uri={}&finInst=fid-smn&response_type=code",
        client_id,
        encode(&redirect_uri)
    );
    open::that(&auth_url).unwrap();

    println!("Waiting for OAuth callback on {redirect_uri}...");

    // Block and wait for the code from server thread
    let code = rx.recv().unwrap();
    println!("Code: {}", code);
    code
}

fn get_access_token(code: String) {
    todo!("Should probably return result");
}

fn refresh_access_token(refresh_token: String) {
    todo!("Should probably return result");
}

fn test_token(access_token: String) -> bool {
    true
}
