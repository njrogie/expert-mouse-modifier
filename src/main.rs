use simplelog::{SimpleLogger, LevelFilter, Config};
use tokio;
use std::{env, path::PathBuf};

mod localdata;
mod cli;

use localdata::DataStorage;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    SimpleLogger::init(LevelFilter::Info, Config::default())?;

    let v_args: Vec<String> = env::args().collect();

    let config_file: PathBuf;
    if v_args.len() > 1 { 
        config_file = v_args[1].clone().into();
    } else {
        // launch cli mode
        let home_dir = Some(std::env::var("HOME_DIR")?.into());
        let data_storage = DataStorage::new(home_dir);
        config_file = cli::choose_file(data_storage);
    }

    expert_mouse_modifier::init(config_file).await?;

    Ok(())
}

