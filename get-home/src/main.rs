use structopt::StructOpt;
use std::path::PathBuf;

#[derive(StructOpt)]
#[structopt(name = "get-home")]
/// Utility for managing home data.
enum GetHome {
    /// Get the latest data available.
    Latest,
    /// Remove all data from the system.
    Clean,
    /// Export all data from the system.
    Export,
    /// Import data into the system.
    Import {
        #[structopt(parse(from_os_str))]
        /// File to import from.
        path: PathBuf
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opt = GetHome::from_args();

    match opt {
        GetHome::Latest => {
            let temp = temperature_tools::last_temp().await?;
            println!("Latest Temperature: {} :: {}C", temp.timestamp, temp.centigrade);
        },
        GetHome::Export => {
            let temps = temperature_tools::all_temps_json().await?;
            println!("{}", temps)
        },
        GetHome::Import{path} => {
            temperature_tools::import_many(&path).await?;
        },
        GetHome::Clean => {
            temperature_tools::clear_all().await?;
        }
    }

    Ok(())
}
