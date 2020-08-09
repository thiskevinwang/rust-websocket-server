use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use std::time::Instant;

use futures::{FutureExt, StreamExt};
use tokio::sync::{mpsc, RwLock};
use tokio_postgres::Error;
use warp::ws::{Message, WebSocket};
use warp::Filter;

#[macro_use]
extern crate log;

mod models;
use models::{Attempt, User};

mod initialize_logger;
use initialize_logger::initialize_logger;

mod create_connection;
use create_connection::create_connection;

/// Our global unique user id counter.
static NEXT_USER_ID: AtomicUsize = AtomicUsize::new(1);

/// Our state of currently connected users.
///
/// - Key is their id
/// - Value is a sender of `warp::ws::Message`
type Users = Arc<RwLock<HashMap<usize, mpsc::UnboundedSender<Result<Message, warp::Error>>>>>;

#[tokio::main]
async fn main() -> Result<(), Error> {
    initialize_logger(log::Level::Debug);
    let client = create_connection().await?;

    // Now we can execute a simple statement that just returns its parameter.

    // tokio::spawn(async move {
    //     info!("THREAD 1");
    //     let now = Instant::now();
    //     let rows = client.query("SELECT * FROM users;", &[]).await;
    //     info!("--- Took {}μs", now.elapsed().as_micros());
    // });

    // tokio::spawn(async move {
    //     info!("THREAD 2");
    //     let now = Instant::now();
    //     let rows = client.query("SELECT * FROM attemps;", &[]).await;
    //     info!("--- Took {}μs", now.elapsed().as_micros());
    // });

    info!("SELECT * FROM users;");
    let now = Instant::now();
    let rows = client
        .query(
            "SELECT * FROM users LIMIT $1 OFFSET $2;",
            &[&None::<i64>, &None::<i64>],
        )
        .await?;
    info!("--- Took {}μs", now.elapsed().as_micros());

    // info!("SELECT * FROM attempts;");
    // let now = Instant::now();
    // let rows = client
    //     .query(
    //         "SELECT * FROM attempts LIMIT $1 OFFSET $2;",
    //         &[&100i64, &100i64],
    //     )
    //     .await?;
    // info!("--- Took {}μs", now.elapsed().as_micros());

    let mut pg_users: Vec<User> = vec![];
    info!("Deserializing {} Rows", rows.len());
    let now = Instant::now();
    for row in rows {
        let user = User::from(row);
        pg_users.push(user);
    }
    info!("--- Took {}μs", now.elapsed().as_micros());

    // And then check that we got back the same string we sent over.
    // let value: &str = rows[0].get(0);

    // Keep track of all connected users, key is usize, value
    // is a websocket sender.
    let users_arc = Users::default();
    let users_arc_2 = users_arc.clone();
    // Turn our "state" into a new Filter...
    let users = warp::any().map(move || users_arc.clone());
    let users_2 = warp::any().map(move || users_arc_2.clone());

    let pg_rows = warp::any().map(move || pg_users.clone());
    // GET /users
    let get_users = warp::path("users").and(pg_rows).map(|u| {
        println!("GET /users, {:?}", u);
        // GET /users, RwLock { s: Semaphore { permits: 64 }, c: UnsafeCell }

        // FIXME
        warp::reply::json(&u)
    });

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

    let routes = index.or(chat).or(health).or(get_users);

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
