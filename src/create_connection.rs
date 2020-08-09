use tokio_postgres::{Client, Error, NoTls};

// static CONNECTION_STRING: &str =
//     "host=localhost port=8080 user=postgres password='mysecretpassword'";
static CONNECTION_STRING: &str = "postgres://postgres:mysecretpassword@localhost:8080/postgres";

pub async fn create_connection() -> Result<Client, Error> {
    // @see https://docs.rs/tokio-postgres/0.5.5/tokio_postgres/config/struct.Config.html#examples
    let (client, connection) = tokio_postgres::connect(CONNECTION_STRING, NoTls).await?;

    // The connection object performs the actual communication with the database,
    // so spawn it off to run on its own.
    tokio::spawn(connection);

    Ok(client)
}
