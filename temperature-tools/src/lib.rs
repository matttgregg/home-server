use chrono::{DateTime, Utc};
use rust_decimal::{prelude::ToPrimitive, Decimal};
use tokio_postgres::{Error, NoTls};

pub struct TimedTemp {
    pub timestamp: DateTime<Utc>,
    pub centigrade: f32,
}

pub async fn all_temps() -> Result<Vec<TimedTemp>, Error> {
    dotenv::dotenv().ok();
    let usr = dotenv::var("HOME_USER");

    let default_conn_string = format!(
        "host='localhost' dbname='home' user='{}'",
        usr.unwrap_or("<<HOME_USER>>".to_string())
    );
    let conn_string = dotenv::var("HOME_CONN").unwrap_or(default_conn_string);

    // Connect to the database.
    let (client, connection) = tokio_postgres::connect(&conn_string, NoTls).await?;

    // The connection object performs the actual communication with the database,
    // so spawn it off to run on its own.
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    let rows = client.query("SELECT * FROM temperatures", &[]).await?;

    let mut temps = vec![];

    for v in &rows {
        let temp: Decimal = v.get(1);

        if let Some(centigrade) = temp.to_f32() {
            temps.push(TimedTemp {
                timestamp: v.get(0),
                centigrade,
            });
        }
    }

    Ok(temps)
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
