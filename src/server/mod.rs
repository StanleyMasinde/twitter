pub mod routes;

use std::io::ErrorKind;

use axum::Router;

pub async fn run(port_param: Option<u16>) {
    let default_port: u16 = 3000;
    let port = port_param.unwrap_or(default_port);
    let address = format!("127.0.0.1:{port}");

    let app = Router::new().nest("/api", routes::api::api_routes());

    let listener = match tokio::net::TcpListener::bind(address)
        .await
        .inspect(|t| println!("Server started on http://{}", t.local_addr().unwrap()))
    {
        Ok(l) => l,
        Err(error) => {
            if error.kind() == ErrorKind::PermissionDenied {
                eprintln!("You don't have persmission to port {port}.")
            } else if error.kind() == ErrorKind::AddrInUse {
                eprintln!("Port {port} is already in use.")
            } else {
                eprintln!("Could not start the server {error}")
            }

            std::process::exit(1)
        }
    };
    axum::serve(listener, app)
        .await
        .expect("Failed to start Axum.");
}
