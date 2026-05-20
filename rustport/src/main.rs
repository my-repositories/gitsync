mod configuration;

use anyhow::Result;
use configuration::config_reader::ConfigReader;

#[tokio::main]
async fn main() -> Result<()> {
    let cfg = ConfigReader::read_config().await?;
    println!("{:#?}", cfg);
    Ok(())
}