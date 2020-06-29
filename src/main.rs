use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};

use std::{convert::Infallible, net::SocketAddr};

/* for hot reloading */
use listenfd::ListenFd;

mod helpers;
use helpers::get_increment_count;

/// Handle GET requests to /
fn get_index(req: &Request<Body>) -> Result<Response<Body>, hyper::Error> {
    Ok(Response::new(Body::from("Try visting /redis")))
}

/// This is our service handler. It receives a Request, routes on its
/// path, and returns a Future of a Response.
async fn echo(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    match (req.method(), req.uri().path()) {
        // Serve some instructions at /
        (&Method::GET, "/") => get_index(&req),

        (&Method::GET, "/redis") => get_increment_count(&req),

        // Return the 404 Not Found for other routes.
        _ => {
            let mut not_found = Response::default();
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
    }
}

/// Run with:
/// - cargo run
/// Run with hot reloading:
/// - systemfd --no-pid -s http::3000 -- cargo watch -x run
/// Find previous task on a port
/// - netstat -vanp tcp | grep 3000    
#[tokio::main]
async fn main() {
    /* for hot reloading */
    let mut listenfd = ListenFd::from_env();

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));

    let make_svc = make_service_fn(|_conn| async { Ok::<_, Infallible>(service_fn(echo)) });

    let server = if let Some(tcp_listener) = listenfd.take_tcp_listener(0).unwrap() {
        println!("→ Hot reloading 🔥");
        println!("http://{}", tcp_listener.local_addr().unwrap());
        Server::from_tcp(tcp_listener).unwrap().serve(make_svc)
    } else {
        println!("→ Starting a new service ✨");
        println!("http://{}", addr);
        Server::bind(&addr).serve(make_svc)
    };

    // And now add a graceful shutdown signal...
    let graceful = server.with_graceful_shutdown(shutdown_signal());

    // Run this server for... forever!
    if let Err(e) = graceful.await {
        eprintln!("server error: {}", e);
    }
}

/// This is needed to avoid lingering processes when using hot reloading.
/// The error:
/// > error: EADDRINUSE: Address already in use
async fn shutdown_signal() {
    // Wait for the CTRL+C signal
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install CTRL+C signal handler");
}
