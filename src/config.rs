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

fn validate_config(config: &Config) -> Result<()> {
    // No two stations can overlap more than the tuning_width
    let mut frequencies: Vec<_> = config.stations.iter().map(|c| c.frequency).collect();
    frequencies.sort_by(|a, b| a.total_cmp(b));
    for (left, right) in frequencies.windows(2).map(|w| (w[0], w[1])) {
        if left + config.tuning_width > right {
            anyhow::bail!("Two stations can't overlap more than the tuning width");
        }
    }
    Ok(())
}

fn parse_config(raw: &str) -> Result<Config> {
    let config = toml::from_str(raw)?;

    validate_config(&config)?;

    Ok(config)
}

pub fn load_config(path: &str) -> Result<Config> {
    let raw_config = std::fs::read_to_string(path)?;
    parse_config(&raw_config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_config_valid() {
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

        let config: Config = parse_config(raw_config).unwrap();
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

    #[test]
    fn validate_config_frequencies_overlap_too_much() {
        let config = Config {
            tuning_width: 0.1,
            white_noise_vol: 0.3,
            station_vol: 1.0,
            stations: vec![
                Station {
                    name: "Foo".into(),
                    frequency: 0.1,
                    path: "".into(),
                    duration: 100.,
                },
                Station {
                    name: "bar".into(),
                    frequency: 0.19,
                    path: "".into(),
                    duration: 100.,
                },
            ],
        };

        let validation_res = validate_config(&config);
        assert!(validation_res.is_err());
        assert!(
            validation_res
                .unwrap_err()
                .to_string()
                .contains("Two stations can't overlap more than the tuning width")
        );
    }
}
