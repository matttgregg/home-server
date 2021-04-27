use chrono::{DateTime, Utc};
use rust_decimal::{prelude::ToPrimitive, Decimal};
use tokio_postgres::NoTls;
use serde::{Serialize, Deserialize};
use anyhow::Result;
use thiserror::Error;
use std::path::PathBuf;
use std::fs::File;
use std::io::BufReader;
use std::convert::TryFrom;

#[derive(Serialize, Deserialize)]
pub struct TimedTemp {
    pub timestamp: DateTime<Utc>,
    pub centigrade: f32,
}

#[derive(Error, Debug)]
pub enum TemperatureError {
    #[error("General temperature error")]
    TemperatureError,

    #[error("IO Error")]
    IOError(String),

    #[error(transparent)]
    DatabaseError(#[from] tokio_postgres::Error)
}

fn temps_to_json(temps: &[TimedTemp]) -> serde_json::Result<String> {
    serde_json::to_string(temps)
}

pub async fn all_temps_json() -> Result<String, TemperatureError> {
    let all_temps = all_temps().await?;
    temps_to_json(&all_temps).map_err(|_| TemperatureError::TemperatureError)
}

pub async fn import_many(path: &PathBuf) -> Result<(), TemperatureError> {

    let file = File::open(path).map_err(|_| TemperatureError::IOError(format!("Couldn't read from {:?}", path)))?;
    let reader = BufReader::new(file);

    // Read the JSON contents of the file as an instance of `User`.
    let temps: Vec<TimedTemp> = serde_json::from_reader(reader).map_err(|_| TemperatureError::IOError("Couldn't deserialize data".to_string()))?;

    let mut client = home_client().await?;
    let trans = client.transaction().await?;
    for temp in temps.into_iter() { 
        let centigrade: Decimal = Decimal::try_from(temp.centigrade).map_err(|_| TemperatureError::TemperatureError)?;
        trans.execute("INSERT INTO temperatures(time, centigrade) VALUES ($1, $2)", &[&temp.timestamp, &centigrade]).await?;
    }

    trans.commit().await?;
    Ok(())
}

pub async fn clear_all() -> Result<(), TemperatureError> {
    let mut client = home_client().await?;
    let trans = client.transaction().await?;
    trans.execute("DELETE FROM temperatures", &[]).await?;
    trans.commit().await?;
    Ok(())
}

async fn home_client<'a>() -> Result<tokio_postgres::Client, TemperatureError> {
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

    Ok(client)
}

pub async fn last_temp() -> Result<TimedTemp, TemperatureError> {
    let client = home_client().await?;
    let rows = client.query("SELECT * FROM temperatures ORDER BY time DESC LIMIT 1", &[]).await?;

    if let Some(latest) = rows.first() {
        let temp: Decimal = latest.get(1);

        if let Some(centigrade) = temp.to_f32() {
            Ok(TimedTemp {
                timestamp: latest.get(0),
                centigrade,
            })
        } else {
            Err(TemperatureError::IOError("could not parse temperature".to_string()))
        }
    } else {
        Err(TemperatureError::IOError("no temperatures found".to_string()))
    }
}

pub async fn all_temps() -> Result<Vec<TimedTemp>, TemperatureError> {
    let client = home_client().await?;
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
