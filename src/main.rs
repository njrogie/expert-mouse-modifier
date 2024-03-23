use simplelog::{SimpleLogger, LevelFilter, Config};
use expert_mouse_modifier;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = SimpleLogger::init(LevelFilter::Info, Config::default());
    let _ = expert_mouse_modifier::init().await;

    Ok(())
}

