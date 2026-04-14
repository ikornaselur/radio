use anyhow::Result;
use rodio::{Decoder, MixerDeviceSink, Player, Source, source::Pink};
use serde::Deserialize;
use std::{
    fs::File,
    io::BufReader,
    num::NonZero,
    time::{Duration, Instant, SystemTime},
};

use crate::Config;

const UNLOAD_STATION_BUFFER_S: f32 = 5.0;
const STATION_TUNING_WIDTH_BUFFER: f32 = 1.1;

#[derive(Deserialize, Clone)]
pub struct Station {
    pub name: String,
    pub path: String,
    pub frequency: f32,
}

struct StationPlayer {
    station: Station,
    player: Option<Player>,
    inactive: Option<Instant>,
}

pub struct StationManager {
    dial: f32,
    volume: f32,
    tuning_width: f32,
    tick_interval: Duration,
    last_tick: Instant,

    station_players: Vec<StationPlayer>,
    static_player: Player,

    sink: MixerDeviceSink,
}

fn load_source(path: &str, seek: bool) -> Result<Decoder<BufReader<File>>> {
    let file = BufReader::new(File::open(path)?);
    let mut source = Decoder::try_from(file)?;

    if seek && let Some(duration) = source.total_duration() {
        let offset = now()? % duration.as_secs_f64();
        source.try_seek(Duration::from_secs_f64(offset))?;
    }

    Ok(source)
}

fn now() -> Result<f64> {
    Ok(SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)?
        .as_secs_f64())
}

impl StationManager {
    #[allow(clippy::missing_errors_doc)]
    pub fn from_config(config: Config) -> Result<Self> {
        let sink = rodio::DeviceSinkBuilder::open_default_sink()?;

        // Set up the white noise
        let static_player = Player::connect_new(sink.mixer());
        static_player.set_volume(0.0);
        let static_sample_rate =
            NonZero::new(48_000).ok_or(anyhow::anyhow!("Unable to create a NonZero"))?;
        let noise = Pink::new(static_sample_rate);
        static_player.append(noise);

        Ok(Self {
            station_players: config
                .stations
                .into_iter()
                .map(|station| StationPlayer {
                    station,
                    player: None,
                    inactive: None,
                })
                .collect(),
            dial: 0.0,
            volume: 0.0,
            tuning_width: config.tuning_width,
            tick_interval: Duration::from_secs_f64(1.0 / 60.0),
            last_tick: Instant::now(),
            sink,
            static_player,
        })
    }

    /// Perform a 'tick'
    ///
    /// In each tick we update the stations and then sleep. A tick consist of:
    ///     * Check if the dial has changed
    ///     * If the dial changed, load stations and mark active one
    ///     * If the dial changed, we update the volume of active stations
    ///     * We unload stations if they've been inactive for (`UNLOAD_STATION_BUFFER_S`) seconds
    ///     * We sleep until the next tick interval
    #[allow(clippy::missing_errors_doc)]
    pub fn tick(&mut self, dial: f32, volume: f32) -> Result<()> {
        if (self.dial - dial).abs() > 0.001 || (self.volume - volume).abs() > 0.001 {
            log::debug!("Dial updated to {dial}, volume updated to {volume}");
            self.dial = dial;
            self.volume = volume;

            let active_station_players = self.load_stations()?;
            self.update_volumes(&active_station_players)?;
        }
        // Always unload stations
        self.unload_stations();

        // Sleep until next tick
        self.last_tick = Instant::now();
        let sleep_time = self.tick_interval.saturating_sub(self.last_tick.elapsed());
        std::thread::sleep(sleep_time);

        Ok(())
    }

    /// Load stations
    ///
    /// Based on the current dial, we load stations that were not previously loaded if they are
    /// within `STATION_TUNING_WIDTH_BUFFER` * `tuning_width`
    ///
    /// This lets us keep adjacent stations ready to play if the dial is turned towards them.
    fn load_stations(&mut self) -> Result<Vec<usize>> {
        let mut active_stations = vec![];

        for (idx, sp) in self.station_players.iter_mut().enumerate() {
            let distance = (self.dial - sp.station.frequency).abs();

            // A station is only active if it's within two tuning widths
            if distance > self.tuning_width * STATION_TUNING_WIDTH_BUFFER {
                if sp.player.is_some() && sp.inactive.is_none() {
                    // Just became inactive, so we'll set the flag for cleanup
                    log::info!(
                        "Station '{}' became inactive, flagging for cleanup",
                        sp.station.name
                    );
                    sp.inactive = Some(Instant::now());
                }

                // If it's outside of the buffer, we just drop the volume down to 0 immediately, if
                // it's not already zero
                // This is to account for dialing real fast around
                if let Some(player) = &sp.player
                    && player.volume() > 0.0
                {
                    player.set_volume(0.0);
                }

                continue;
            }
            // Station has become active again, so we'll remove the flag
            if sp.inactive.is_some() {
                sp.inactive = None;
            }

            active_stations.push(idx);
            // If we're already loaded, we just continue
            if sp.player.is_some() {
                continue;
            }
            log::info!("Loading station '{}'", sp.station.name);

            let player = Player::connect_new(self.sink.mixer());
            player.set_volume(0.0);
            player.append(load_source(&sp.station.path, true)?);
            sp.player = Some(player);
        }

        Ok(active_stations)
    }

    /// Unload stations
    ///
    /// If a loaded station is outside the `STATION_TUNING_WIDTH_BUFFER` * `tuning_width`, then we
    /// start unloading them. To prevent thrashing if you move the dial quickly back and forth we
    /// keep stations in memory for `UNLOAD_STATION_BUFFER_S` seconds.
    fn unload_stations(&mut self) {
        for sp in &mut self.station_players {
            if let Some(instant) = sp.inactive
                && instant.elapsed() > Duration::from_secs_f32(UNLOAD_STATION_BUFFER_S)
            {
                log::info!("Cleaning up station '{}'", sp.station.name);
                sp.player = None;
                sp.inactive = None;
            }
        }
    }

    /// Get the station volume
    ///
    /// The volume scales linearly from the edge of the `tuning_width` towards 10% of the
    /// `tuning_width`, then the volume is stable at full volume for the last 10%
    /// This means that the inner 10% of the tuning width is "stable full volume"
    fn get_station_volume(&self, station: &Station) -> f32 {
        let distance = (self.dial - station.frequency).abs();
        ((self.tuning_width - distance) / (self.tuning_width - (self.tuning_width * 0.1)))
            .clamp(0.0, 1.0)
    }

    /// Update the volume of stations
    ///
    /// This is called if the dial has moved, so that we can increase or lower volume of stations
    /// and white noise
    fn update_volumes(&self, active_stations: &[usize]) -> Result<()> {
        let mut static_vol: f32 = 1.0;

        for sp in active_stations
            .iter()
            .map(|idx| &self.station_players[*idx])
        {
            let Some(player) = &sp.player else {
                log::warn!("Tried to update volume on a non-existing player");
                continue;
            };

            let station_vol = self.get_station_volume(&sp.station);

            player.set_volume(station_vol * self.volume);
            static_vol = static_vol.min(1.0 - station_vol);

            if player.empty() {
                player.append(load_source(&sp.station.path, false)?);
            }
        }

        self.static_player.set_volume(static_vol * self.volume);

        Ok(())
    }
}
