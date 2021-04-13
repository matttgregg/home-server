use structopt::StructOpt;
use std::path::PathBuf;

#[derive(StructOpt)]
#[structopt(name = "get-home")]
/// Utility for managing home data.
enum GetHome {
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
        GetHome::Export => {
            let temps = temperature_tools::all_temps().await?;
            for t in &temps {
                println!("{} {} C", t.timestamp, t.centigrade);
            }
        },
        _ => {
            unimplemented!()
        }
    }

    Ok(())
}
