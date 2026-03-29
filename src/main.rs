use anyhow::Result;
use device_query::{DeviceQuery, DeviceState};
use radio::config::load_config;
use rodio::{Decoder, Player, Source, source::noise::WhiteUniform};
use std::{
    fs::File,
    io::BufReader,
    num::NonZero,
    thread,
    time::{Duration, SystemTime},
};

fn main() -> Result<()> {
    // Mouse prototype
    let device_state = DeviceState::new();

    // Config
    let config = load_config("./config.toml")?;
    let stations = config.stations;

    /*
     * Audio players
     */
    let handle = rodio::DeviceSinkBuilder::open_default_sink().expect("open default audio stream");

    // Set up white noise
    let white_noise_player = Player::connect_new(handle.mixer());
    white_noise_player.set_volume(0.5);
    let sample_rate = NonZero::new(44100).unwrap();
    let white_noise = WhiteUniform::new(sample_rate);
    white_noise_player.append(white_noise);

    // Get the total duration for each to do offsets
    let mut totals = vec![];
    for station in &stations {
        let buf = Decoder::try_from(BufReader::new(File::open(&station.path)?))?;
        let duration = buf.total_duration().unwrap();
        totals.push(duration);
    }

    /*
     * Set up stations
     */
    let mut players = vec![];

    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)?
        .as_secs_f64();

    for (station, station_duration) in stations.iter().zip(totals) {
        let player = Player::connect_new(handle.mixer());
        player.set_volume(0.0);
        let file = BufReader::new(File::open(&station.path)?);
        let mut source = Decoder::try_from(file)?;
        let offset = now % station_duration.as_secs_f64();
        source.try_seek(Duration::from_secs_f64(offset))?;
        player.append(source);
        players.push(player);
        println!("Offsetting {}: {:?}", station.path, offset);
    }

    /*
     * The main loop
     */
    let mut last_tuned_station = String::new();
    let mut last_dial = 0.;
    loop {
        let (x, _) = device_state.get_mouse().coords;
        let dial = (x as f32 / 1800f32).clamp(0.0, 1.0);
        if dial == last_dial {
            thread::sleep(Duration::from_millis((1000f32 / 60f32) as u64));
            continue;
        }
        last_dial = dial;

        // Let's just update all stations for now, it's naive and inefficient, but for testing it
        // works fine.
        let mut white_noise_vol: f32 = 1.0;
        for (station, player) in stations.iter().zip(&players) {
            let distance = (dial - station.frequency).abs();
            let audio_vol = (1.0 - distance / config.tuning_width).clamp(0.0, 1.0);
            let noise_vol = 1.0 - audio_vol;

            if audio_vol > 0.9 && station.name != last_tuned_station {
                last_tuned_station = station.name.clone();
                println!("Tuned into {}", last_tuned_station);
            }

            player.set_volume(audio_vol);
            white_noise_vol = white_noise_vol.min(noise_vol);
            // Wrap around to the start when at the end
            if player.empty() {
                println!("Reloading {}", station.path);
                let file = BufReader::new(File::open(&station.path)?);
                let source = Decoder::try_from(file)?;
                player.append(source);
            }
        }
        white_noise_player.set_volume(white_noise_vol * config.white_noise_vol);

        thread::sleep(Duration::from_millis((1000f32 / 60f32) as u64));
    }

    // Ok(())
}
