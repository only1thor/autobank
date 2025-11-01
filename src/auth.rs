use std::thread;
use tiny_http::{Response, Server};
use url::form_urlencoded;
use urlencoding::encode;

pub fn auth(client_id: String) -> String {
    // Pick a port
    let port = 8321;
    let redirect_uri = format!("http://localhost:{}", port);

    // Start server in a background thread
    let server = Server::http(format!("127.0.0.1:{port}")).unwrap();

    thread::spawn(move || {
        for request in server.incoming_requests() {
            if true {
                let query = request.url().split('?').nth(1).unwrap_or("");
                let params: std::collections::HashMap<_, _> =
                    form_urlencoded::parse(query.as_bytes())
                        .into_owned()
                        .collect();

                let body = "✅ You can close this tab now.";
                let response = Response::from_string(body);
                let _ = request.respond(response);

                if let Some(code) = params.get("code") {
                    println!("Received code: {}", code);
                    std::process::exit(0);
                }
            }
        }
    });

    // Build OAuth URL
    let path = format!(
        "https://api.sparebank1.no/oauth/authorize?client_id={}&state=1226754&redirect_uri={}&finInst=fid-smn&response_type=code",
        client_id, &redirect_uri
    );

    // Open browser
    match open::that(&path) {
        Ok(()) => println!("Opened '{}' successfully.", path),
        Err(err) => eprintln!("Failed to open '{}': {}", path, err),
    }

    // Block until the process exits (or replace with channel/flag if you want to return the code)
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
