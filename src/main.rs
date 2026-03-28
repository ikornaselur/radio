use anyhow::Result;
use device_query::{DeviceQuery, DeviceState};
use rodio::{Decoder, Player, source::noise::WhiteUniform};
use std::{fs::File, io::BufReader, num::NonZero, thread, time::Duration};

struct Station<'a> {
    path: &'a str,
    frequency: f32,
}

const TUNING_WIDTH: f32 = 0.1;
const WHITE_NOISE_VOL: f32 = 0.3;

const CHANNEL_1: Station = Station {
    path: "./channels/channel1.mp3",
    frequency: 0.3,
};
const CHANNEL_2: Station = Station {
    path: "./channels/channel2.mp3",
    frequency: 0.55,
};
const CHANNEL_3: Station = Station {
    path: "./channels/channel3.mp3",
    frequency: 0.75,
};
const STATIONS: [Station; 3] = [CHANNEL_1, CHANNEL_2, CHANNEL_3];

fn main() -> Result<()> {
    /*
     * Mouse prototype
     */
    let device_state = DeviceState::new();
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

    // Set up stations
    let mut players = vec![];

    for station in STATIONS {
        let player = Player::connect_new(handle.mixer());
        player.set_volume(0.0);
        let file = BufReader::new(File::open(station.path)?);
        let source = Decoder::try_from(file)?;
        player.append(source);

        players.push(player);
    }

    loop {
        let (x, _) = device_state.get_mouse().coords;
        let dial = (x as f32 / 1800f32).clamp(0.0, 1.0);

        // Let's just update all stations for now, it's naive and inefficient, but for testing it
        // works fine.
        let mut white_noise_vol: f32 = 1.0;
        for (station, player) in STATIONS.iter().zip(&players) {
            let distance = (dial - station.frequency).abs();
            let audio_vol = (1.0 - distance / TUNING_WIDTH).clamp(0.0, 1.0);
            let noise_vol = 1.0 - audio_vol;

            player.set_volume(audio_vol);
            white_noise_vol = white_noise_vol.min(noise_vol);
        }
        white_noise_player.set_volume(white_noise_vol * WHITE_NOISE_VOL);

        thread::sleep(Duration::from_millis((1000f32 / 60f32) as u64));
    }

    // Ok(())
}
