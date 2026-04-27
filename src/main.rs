use anyhow::Result;
use radio::{ADCReader, StationManager, load_config};

fn main() -> Result<()> {
    env_logger::init();

    let config = load_config("./config.toml")?;

    let mut manager: StationManager = StationManager::from_config(config)?;
    let adc_reader = ADCReader::spawn()?;

    loop {
        // Tick will internally sleep
        manager.tick(adc_reader.dial(), adc_reader.volume())?;
    }
}
