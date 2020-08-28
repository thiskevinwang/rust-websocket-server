use std::collections::HashMap;
use std::convert::Infallible;
use std::str::FromStr;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use std::time::Instant;

use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use chrono::NaiveDateTime;
use futures::{FutureExt, StreamExt};
use serde::Deserialize;
use tokio::sync::{mpsc, RwLock};
use tokio_postgres::{Error, NoTls, Row};
use uuid::Uuid;
use warp::ws::{Message, WebSocket};
use warp::Filter;

#[macro_use]
extern crate log;

mod models;
use models::{Attempt, User};

mod initialize_logger;
use initialize_logger::initialize_logger;

/// Our global unique user id counter.
static NEXT_USER_ID: AtomicUsize = AtomicUsize::new(1);

/// Our state of currently connected users.
///
/// - Key is their id
/// - Value is a sender of `warp::ws::Message`
type Users = Arc<RwLock<HashMap<usize, mpsc::UnboundedSender<Result<Message, warp::Error>>>>>;

#[derive(Deserialize)]
struct Options {
    limit: Option<i64>,
    offset: Option<i64>,
}

#[derive(Deserialize)]
struct Range {
    start: Option<i64>,
    end: Option<i64>,
}

/// # with_pool
/// Taken from https://blog.logrocket.com/create-an-async-crud-web-service-in-rust-with-warp/
///
/// @usage
/// ```
/// let get_users = warp::path!("users")
/// .and(with_pool(pool.clone()))
/// .and(warp::query::<Options>())
/// .map(
///     |pool: Pool<PostgresConnectionManager<NoTls>>, opts: Options| {
///         info!("GET /users");
///         warp::reply::json(&1)
///     },
/// );
/// ```
fn with_pool(
    pool: Pool<PostgresConnectionManager<NoTls>>,
) -> impl Filter<Extract = (Pool<PostgresConnectionManager<NoTls>>,), Error = std::convert::Infallible>
       + Clone {
    warp::any().map(move || pool.clone())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    initialize_logger(log::Level::Debug);

    // ------------------------------------------------------------------------
    let config = tokio_postgres::config::Config::from_str(
        "postgres://postgres:mysecretpassword@localhost:8080/postgres",
    )
    .unwrap();
    let pg_mgr = PostgresConnectionManager::new(config, tokio_postgres::NoTls);
    let pool = match Pool::builder().build(pg_mgr).await {
        Ok(pool) => pool,
        Err(e) => panic!("builder error: {:?}", e),
    };

    // And then check that we got back the same string we sent over.
    // let value: &str = rows[0].get(0);

    // Keep track of all connected users, key is usize, value
    // is a websocket sender.
    let users_arc = Users::default();
    // Turn our "state" into a new Filter...
    let users = warp::any().map(move || users_arc.clone());

    // GET /users
    async fn async_query_users(
        pool: Pool<PostgresConnectionManager<NoTls>>,
        opts: Options,
    ) -> Result<impl warp::Reply, Infallible> {
        info!("GET /users");
        let limit = if let Some(_limit) = opts.limit {
            Some(_limit)
        } else {
            None::<i64>
        };
        let offset = match opts.offset {
            Some(_offset) => Some(_offset),
            None::<i64> => None::<i64>,
        };
        let conn = pool.get().await.unwrap();
        let rows = conn
            .query(
                "SELECT * FROM users LIMIT $1 OFFSET $2;",
                &[&limit, &offset],
            )
            .await
            .unwrap();
        let mut users_array: Vec<User> = vec![];
        for row in rows {
            let user = User::from(Row::from(row));
            users_array.push(user);
        }

        Ok(warp::reply::json(&users_array))
    };

    /// # GET /users/:userId/attempts
    /// QUERY STRING PARAMS
    /// - `limit=15`, `offset=15`, `start=1596192327`, `end=1597192327`
    /// - `/users/a5f5d36a-6677-41c2-85b8-7578b4d98972/attempts?limit=15&offset=15&start=1596192327&end=1597192327`
    async fn async_query_attempts_for_user(
        user_id: Uuid,
        pool: Pool<PostgresConnectionManager<NoTls>>,
        opts: Options,
        range: Range,
    ) -> Result<impl warp::Reply, Infallible> {
        info!("async_query_attempts_for_user");
        let now = Instant::now();
        let limit = if let Some(_limit) = opts.limit {
            Some(_limit)
        } else {
            None::<i64>
        };
        let offset = match opts.offset {
            Some(_offset) => Some(_offset),
            None::<i64> => None::<i64>,
        };

        let (start, end) = match (range.start, range.end) {
            (Some(start_s), Some(end_s)) => {
                // convert js date as number (milliseconds)
                // to rust NaiveDateTime timestamp (seconds)
                let start = NaiveDateTime::from_timestamp(start_s / 1000, 0);
                let end = NaiveDateTime::from_timestamp(end_s / 1000, 0);
                (Some(start), Some(end))
            }
            _ => (None, None),
        };

        info!("{:?}...{:?}", start, end);
        let conn = pool.get().await.unwrap();
        let rows = conn
            .query(
                "
            SELECT *
            FROM attempts a
	        LEFT 
            JOIN users u
                ON u.id = a.user_id
            WHERE a.user_id = u.id
                AND u.id = $1
                AND a.date BETWEEN SYMMETRIC $2 AND $3
            ORDER BY date ASC
            LIMIT $4
            OFFSET $5;
            ",
                &[&user_id, &start, &end, &limit, &offset],
            )
            .await
            .unwrap();
        let mut attempts_array: Vec<Attempt> = vec![];
        for row in rows {
            let attempt = Attempt::from(Row::from(row));
            attempts_array.push(attempt);
        }

        info!("{}μs", now.elapsed().as_micros());
        Ok(warp::reply::json(&attempts_array))
    };

    async fn async_query_attempts(
        pool: Pool<PostgresConnectionManager<NoTls>>,
        opts: Options,
    ) -> Result<impl warp::Reply, Infallible> {
        let limit = if let Some(_limit) = opts.limit {
            Some(_limit)
        } else {
            None::<i64>
        };
        let offset = match opts.offset {
            Some(_offset) => Some(_offset),
            None::<i64> => None::<i64>,
        };

        info!("Limit: {:?}, Offset: {:?}", limit, offset);

        let conn = pool.get().await.unwrap();
        let rows = conn
            .query(
                "SELECT * FROM attempts LIMIT $1 OFFSET $2;",
                &[&limit, &offset],
            )
            .await
            .unwrap();
        let mut attempts_array: Vec<Attempt> = vec![];
        for row in rows {
            let attempt = Attempt::from(Row::from(row));
            attempts_array.push(attempt);
        }

        // https://docs.rs/warp/0.2.4/warp/reply/fn.json.html
        Ok(warp::reply::json(&attempts_array))
    }

    let get_users = warp::path!("users")
        .and(with_pool(pool.clone()))
        .and(warp::query::<Options>())
        .and_then(async_query_users);

    let get_attempts = warp::path!("attempts")
        .and(with_pool(pool.clone()))
        .and(warp::query::<Options>())
        .and_then(async_query_attempts);

    let cors = warp::cors().allow_any_origin();
    let get_attempts_for_user = warp::path!("users" / Uuid / "attempts")
        .and(with_pool(pool.clone()))
        .and(warp::query::<Options>())
        .and(warp::query::<Range>())
        .and_then(async_query_attempts_for_user)
        .with(cors);

    // GET /chat -> websocket upgrade
    let chat = warp::path("chat")
        // The `ws()` filter will prepare Websocket handshake...
        .and(warp::ws())
        .and(users)
        .map(|ws: warp::ws::Ws, users| {
            println!("GET /chat");
            // This will call our function if the handshake succeeds.
            ws.on_upgrade(move |socket| user_connected(socket, users))
        });

    // GET / -> index html
    let index = warp::path::end().map(|| {
        println!("GET /");
        warp::reply::html(INDEX_HTML)
    });
    let health = warp::path("health").map(|| {
        println!("GET /health");
        warp::reply::json(&"ok".to_string())
    });

    let routes = index
        .or(chat)
        .or(health)
        .or(get_users)
        .or(get_attempts)
        .or(get_attempts_for_user);

    warp::serve(routes).run(([0, 0, 0, 0], 3000)).await;
    Ok(())
}

async fn user_connected(ws: WebSocket, users: Users) {
    // Use a counter to assign a new unique ID for this user.
    let my_id = NEXT_USER_ID.fetch_add(1, Ordering::Relaxed);

    eprintln!("new chat user: {}", my_id);

    // Split the socket into a sender and receive of messages.
    let (user_ws_tx, mut user_ws_rx) = ws.split();

    // Use an unbounded channel to handle buffering and flushing of messages
    // to the websocket...
    let (tx, rx) = mpsc::unbounded_channel();
    tokio::task::spawn(rx.forward(user_ws_tx).map(|result| {
        if let Err(e) = result {
            eprintln!("websocket send error: {}", e);
        }
    }));

    // Save the sender in our list of connected users.
    users.write().await.insert(my_id, tx);

    // Return a `Future` that is basically a state machine managing
    // this specific user's connection.

    // Make an extra clone to give to our disconnection handler...
    let users2 = users.clone();

    // Every time the user sends a message, broadcast it to
    // all other users...
    while let Some(result) = user_ws_rx.next().await {
        let msg = match result {
            Ok(msg) => msg,
            Err(e) => {
                eprintln!("websocket error(uid={}): {}", my_id, e);
                break;
            }
        };
        user_message(my_id, msg, &users).await;
    }

    // user_ws_rx stream will keep processing as long as the user stays
    // connected. Once they disconnect, then...
    user_disconnected(my_id, &users2).await;
}

async fn user_message(my_id: usize, msg: Message, users: &Users) {
    // Skip any non-Text messages...
    let msg = if let Ok(s) = msg.to_str() {
        println!("{:?}", s);
        s
    } else {
        return;
    };

    let new_msg = format!("<User#{}>: {}", my_id, msg);

    // New message from this user, send it to everyone else (except same uid)...
    for (&uid, tx) in users.read().await.iter() {
        if my_id != uid {
            if let Err(_disconnected) = tx.send(Ok(Message::text(new_msg.clone()))) {
                // The tx is disconnected, our `user_disconnected` code
                // should be happening in another task, nothing more to
                // do here.
            }
        }
    }
}

async fn user_disconnected(my_id: usize, users: &Users) {
    eprintln!("good bye user: {}", my_id);

    // Stream closed up, so remove from the user list
    users.write().await.remove(&my_id);
}

static INDEX_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
    <head>
        <title>Warp Chat</title>
    </head>
    <body>
        <h1>Warp chat</h1>
        <div id="chat">
            <p><em>Connecting...</em></p>
        </div>
        <input type="text" id="text" />
        <button type="button" id="send">Send</button>
        <script type="text/javascript">
        const chat = document.getElementById('chat');
        const text = document.getElementById('text');
        const uri = 'ws://' + location.host + '/chat';
        const ws = new WebSocket(uri);
        function message(data) {
            const line = document.createElement('p');
            line.innerText = data;
            chat.appendChild(line);
        }
        ws.onopen = function() {
            chat.innerHTML = '<p><em>Connected!</em></p>';
        };
        ws.onmessage = function(msg) {
            message(msg.data);
        };
        ws.onclose = function() {
            chat.getElementsByTagName('em')[0].innerText = 'Disconnected!';
        };
        send.onclick = function() {
            const msg = text.value;
            ws.send(msg);
            text.value = '';
            message('<You>: ' + msg);
        };
        </script>
    </body>
</html>
"#;
