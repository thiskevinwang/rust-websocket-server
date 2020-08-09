use tokio_postgres::{Client, Error, NoTls};

pub async fn create_connection() -> Result<Client, Error> {
    // @see https://docs.rs/tokio-postgres/0.5.5/tokio_postgres/config/struct.Config.html#examples
    let (client, connection) = tokio_postgres::connect(
        "host=localhost port=8080 user=postgres password='mysecretpassword'",
        NoTls,
    )
    .await?;

    info!("Client: {:?}", client);

    // The connection object performs the actual communication with the database,
    // so spawn it off to run on its own.
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            error!("connection error: {}", e);
        }
    });

    Ok(client)
}
