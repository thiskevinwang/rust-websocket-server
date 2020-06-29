use hyper::{Body, Request, Response};
use redis::{Commands, Connection};
use std::env;

fn increment_count() -> redis::RedisResult<isize> {
    let host = env::var("REDIS_HOST").unwrap_or("localhost".to_string());

    // connect to redis
    let redis_connection_params = format!("redis://{host}:6379", host = host);
    let client = redis::Client::open(redis_connection_params)?;
    let mut con: Connection = client.get_connection()?;
    // throw away the result, just make sure it does not fail

    con.incr("count", 1)
}

pub fn get_increment_count(req: &Request<Body>) -> Result<Response<Body>, hyper::Error> {
    let val = increment_count().unwrap();
    let method = req.method();
    let uri = req.uri();

    Ok(Response::new(
        format!(
            "{method} {uri} {val}",
            method = method,
            uri = uri,
            val = val
        )
        .into(),
    ))
}
