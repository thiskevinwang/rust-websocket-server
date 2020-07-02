use redis::aio::MultiplexedConnection;
use redis::{Commands, Connection, ConnectionAddr, ConnectionInfo};
use std::env;
use warp::Filter;

mod filters;
mod handlers;
mod models;

/// Provides a RESTful web server managing some Todos.
///
/// API will be:
///
/// - `GET /todos`: return a JSON list of Todos.
/// - `POST /todos`: create a new Todo.
/// - `PUT /todos/:id`: update a specific Todo.
/// - `DELETE /todos/:id`: delete a specific Todo.
#[tokio::main]
async fn main() {
    let redis_host = env::var("REDIS_HOST").unwrap_or("localhost".to_string());
    let redis_connection_params = format!("redis://{}:6379", redis_host);

    // ⚠️ ConnectionAddr::Tcp doesn't need the 'redis://' prefix/protocol
    let addr = ConnectionAddr::Tcp(format!("{}", redis_host), 6379);
    let connection_info = ConnectionInfo {
        addr: Box::new(addr),
        db: 0,
        passwd: None,
    };
    let client = redis::Client::open(redis_connection_params).unwrap();
    // let mut connection = client.get_multiplexed_async_std_connection().await.unwrap();

    // let res: redis::RedisResult<isize> = connection.incr("count", 1);
    // println!("res = {}", res.unwrap());

    if env::var_os("RUST_LOG").is_none() {
        // Set `RUST_LOG=todos=debug` to see debug logs,
        // this only shows access logs.
        env::set_var("RUST_LOG", "todos=info");
    }
    pretty_env_logger::init();

    let db = models::blank_db();

    let api = filters::todos(db, connection_info);

    // View access logs by setting `RUST_LOG=todos`.
    let routes = api.with(warp::log("todos"));
    // Start up the server...
    warp::serve(routes).run(([0, 0, 0, 0], 3000)).await;
}
