use std::time::Duration;

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    terminal::enable_raw_mode,
};
use radio::{StationManager, load_config};

const DIAL_STEP: f32 = 0.01;

#[allow(clippy::cast_precision_loss)]
fn main() -> Result<()> {
    env_logger::init();

    let config = load_config("./config.toml")?;

    let mut manager: StationManager = StationManager::from_config(config)?;

    enable_raw_mode()?;

    let mut dial = 0.5;
    loop {
        let mut left_held = false;
        let mut right_held = false;
        let mut break_out = false;
        while event::poll(Duration::ZERO)? {
            match event::read()? {
                Event::Key(KeyEvent {
                    code: KeyCode::Left,
                    ..
                }) => left_held = true,
                Event::Key(KeyEvent {
                    code: KeyCode::Right,
                    ..
                }) => right_held = true,
                Event::Key(KeyEvent {
                    code: KeyCode::Char('q'),
                    modifiers,
                    ..
                })
                | Event::Key(KeyEvent {
                    code: KeyCode::Char('c'),
                    modifiers,
                    ..
                }) if modifiers.contains(KeyModifiers::CONTROL) => break_out = true,
                _ => {}
            }
        }
        if break_out {
            return Ok(());
        }
        let mut dial_delta = 0.0;
        if left_held {
            dial_delta -= DIAL_STEP;
        }
        if right_held {
            dial_delta += DIAL_STEP;
        }

        dial += dial_delta;

        manager.tick(dial)?;
    }
}
