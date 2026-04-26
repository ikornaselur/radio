use ads1x1x::{Ads1x1x, DataRate16Bit, FullScaleRange, TargetAddr, channel};
use anyhow::Result;
use linux_embedded_hal::I2cdev;
use radio::{StationManager, load_config};

const MAX_DIAL: f32 = 26400.0;

#[allow(clippy::cast_precision_loss)]
fn main() -> Result<()> {
    env_logger::init();

    let config = load_config("./config.toml")?;

    let mut manager: StationManager = StationManager::from_config(config)?;

    let dev = I2cdev::new("/dev/i2c-1")?;
    let mut adc = Ads1x1x::new_ads1115(dev, TargetAddr::default());
    adc.set_full_scale_range(FullScaleRange::Within4_096V)
        .unwrap();
    adc.set_data_rate(DataRate16Bit::Sps128).unwrap();
    let mut adc = adc
        .into_continuous()
        .map_err(|_| anyhow::anyhow!("failed to enter continuous mode"))?;
    adc.select_channel(channel::SingleA0).unwrap();

    let volume = 0.5;
    loop {
        // volume = (volume + volume_delta).clamp(0.0, 1.0);
        // dial = (dial + dial_delta).clamp(0.0, 1.0);
        let pot = adc.read().unwrap();
        let dial = (pot as f32) / MAX_DIAL;

        manager.tick(dial, volume)?;
    }
}
