use redis::{Commands, Connection};
use std::env;

pub fn fetch_an_integer() -> redis::RedisResult<isize> {
    println!("attempting to fetch integer");

    let host = env::var("REDIS_HOST").unwrap_or("localhost".to_string());
    println!("Host: {}", host);

    // connect to redis
    let redis_connection_params = format!("redis://{host}:6379", host = host);
    let client = redis::Client::open(redis_connection_params)?;
    let mut con: Connection = client.get_connection()?;
    // throw away the result, just make sure it does not fail

    // Note - currently the browser will increment Redis twice.
    // This is likely because the browser is make two requests;
    // One for '/' and one for a favicon. The current code doe
    // not differentiate between the two yet.
    println!("incrementing");
    con.incr("count", 1)
}
