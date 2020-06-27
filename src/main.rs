use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use redis::{Commands, Connection};
use std::{convert::Infallible, net::SocketAddr};

/* for hot reloading */
use listenfd::ListenFd;

async fn handle(_: Request<Body>) -> Result<Response<Body>, Infallible> {
    let val = fetch_an_integer();
    println!("{:?}", val);
    Ok(Response::new(
        format!("Hello world2, {:?}", val.unwrap()).into(),
    ))
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

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    let make_svc = make_service_fn(|_conn| async { Ok::<_, Infallible>(service_fn(handle)) });

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

const REDIS_HOST: &str = "redis://127.0.0.1:6379";

fn fetch_an_integer() -> redis::RedisResult<isize> {
    // connect to redis
    let client = redis::Client::open(REDIS_HOST)?;
    let mut con: Connection = client.get_connection()?;
    // throw away the result, just make sure it does not fail

    // Note - currently the browser will increment Redis twice.
    // This is likely because the browser is make two requests;
    // One for '/' and one for a favicon. The current code doe
    // not differentiate between the two yet.
    println!("incrementing");
    con.incr("count", 1)
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
