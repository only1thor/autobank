use std::collections::HashMap;
use std::sync::mpsc;
use tiny_http::{Response, Server};
use url::form_urlencoded;
use urlencoding::encode;

pub fn auth(client_id: String) -> String {
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
    code
}

fn main() {
    let client_id = "my_client_id".to_string();
    let code = auth(client_id);
    println!("Got auth code: {}", code);
}
