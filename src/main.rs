use anyhow::Result;
use device_query::{DeviceQuery, DeviceState};
use radio::{StationManager, load_config};

const SCREEN_WIDTH: f32 = 3840.0;

fn main() -> Result<()> {
    let config = load_config("./config.toml")?;

    let mut manager: StationManager = StationManager::from_config(config)?;

    // TODO: Replace device state mouse debugging with a real dial
    let device_state = DeviceState::new();
    loop {
        let (x, _) = device_state.get_mouse().coords;
        let dial = (x as f32 / SCREEN_WIDTH).clamp(0.0, 1.0);

        manager.tick(dial)?;
    }
}
