use simplelog::{SimpleLogger, LevelFilter, Config};
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    SimpleLogger::init(LevelFilter::Info, Config::default())?;
    expert_mouse_modifier::init().await?;

    Ok(())
}

