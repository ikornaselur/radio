use crate::station::Station;
use anyhow::Result;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub tuning_width: f32,
    pub white_noise_vol: f32,
    pub station_vol: f32,
    pub stations: Vec<Station>,
}

pub fn load_config(path: &str) -> Result<Config> {
    let raw_config = std::fs::read_to_string(path)?;
    Ok(toml::from_str(&raw_config)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_valid_config() {
        let raw_config = r#"
            tuning_width = 0.05
            white_noise_vol = 0.3
            station_vol = 1.0

            [[stations]]
            name = "Foo"
            path = "/path/to/foo.mp3"
            frequency = 0.3
            duration = 123.4

            [[stations]]
            name = "Bar"
            path = "/path/to/bar.mp3"
            frequency = 0.7
            duration = 10.0
        "#;

        let config: Config = toml::from_str(raw_config).unwrap();
        assert_eq!(config.stations.len(), 2);

        assert_eq!(config.stations[0].name, "Foo");
        assert_eq!(config.stations[0].frequency, 0.3);
        assert_eq!(config.stations[0].duration, 123.4);

        assert_eq!(config.stations[1].name, "Bar");
        assert_eq!(config.stations[1].frequency, 0.7);
        assert_eq!(config.stations[1].duration, 10.0);

        assert_eq!(config.tuning_width, 0.05);
        assert_eq!(config.white_noise_vol, 0.3);
        assert_eq!(config.station_vol, 1.0);
    }
}
