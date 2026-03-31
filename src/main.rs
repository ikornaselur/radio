use anyhow::Result;
use device_query::{DeviceQuery, DeviceState};
use display_info::DisplayInfo;
use radio::{StationManager, load_config};

#[allow(clippy::cast_precision_loss)]
fn main() -> Result<()> {
    env_logger::init();

    let config = load_config("./config.toml")?;

    let mut manager: StationManager = StationManager::from_config(config)?;

    // We're just going to grab the first display
    let screen_width = DisplayInfo::all()?[0].width as f32;
    log::debug!("Screen width: {screen_width}");
    let device_state = DeviceState::new();
    loop {
        let (x, _) = device_state.get_mouse().coords;
        let dial = (x as f32 / screen_width).clamp(0.0, 1.0);

        manager.tick(dial)?;
    }
}
