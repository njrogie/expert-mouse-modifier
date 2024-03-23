use simplelog::{SimpleLogger, LevelFilter, Config};
use expert_mouse_modifier;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = SimpleLogger::init(LevelFilter::Info, Config::default());
    expert_mouse_modifier::init();

    Ok(())
}

