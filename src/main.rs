use std::time::Duration;

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use radio::{StationManager, load_config};

const DIAL_STEP: f32 = 0.01;
const VOL_STEP: f32 = 0.01;

#[allow(clippy::cast_precision_loss)]
fn main() -> Result<()> {
    env_logger::init();

    let config = load_config("./config.toml")?;

    let mut manager: StationManager = StationManager::from_config(config)?;

    enable_raw_mode()?;

    let mut dial = 0.5;
    let mut volume = 1.0;
    loop {
        let mut should_quit = false;
        let mut volume_delta = 0.0;
        let mut dial_delta = 0.0;

        while event::poll(Duration::ZERO)? {
            if let Event::Key(KeyEvent {
                code, modifiers, ..
            }) = event::read()?
            {
                match code {
                    KeyCode::Left => dial_delta -= DIAL_STEP,
                    KeyCode::Right => dial_delta += DIAL_STEP,
                    KeyCode::Up => volume_delta += VOL_STEP,
                    KeyCode::Down => volume_delta -= VOL_STEP,
                    KeyCode::Char('q') => should_quit = true,
                    KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => {
                        should_quit = true;
                    }
                    _ => {}
                }
            }
        }
        if should_quit {
            break;
        }

        volume = (volume + volume_delta).clamp(0.0, 1.0);
        dial = (dial + dial_delta).clamp(0.0, 1.0);

        manager.tick(dial, volume)?;
    }
    disable_raw_mode()?;
    Ok(())
}
