use ads1x1x::{Ads1x1x, DataRate16Bit, FullScaleRange, TargetAddr, channel};
use anyhow::Result;
use linux_embedded_hal::I2cdev;
use radio::{StationManager, load_config};

// Since the scale range is for ~4.096V and we use 3.3V as reference, we need to target ~26400 as
// the max value we'll read from the potentiometer;
const MAX_DIAL: f32 = (3300.0 / 4096.0) * 32768.0;

enum ADCChannel {
    Dial,
    Volume,
}

#[allow(clippy::cast_precision_loss)]
fn main() -> Result<()> {
    env_logger::init();

    let config = load_config("./config.toml")?;

    let mut manager: StationManager = StationManager::from_config(config)?;

    let dev = I2cdev::new("/dev/i2c-1")?;
    let mut adc = Ads1x1x::new_ads1115(dev, TargetAddr::default());
    adc.set_full_scale_range(FullScaleRange::Within4_096V)
        .unwrap();
    adc.set_data_rate(DataRate16Bit::Sps475).unwrap();
    let mut adc = adc
        .into_continuous()
        .map_err(|_| anyhow::anyhow!("failed to enter continuous mode"))?;

    adc.select_channel(channel::SingleA0).unwrap();

    let mut volume = 0.0;
    let mut dial = 0.0;
    let mut current_channel = ADCChannel::Dial;

    // We'll alternate between rading each ADC channel, giving it a full tick to populate (?) in
    // the background

    loop {
        match current_channel {
            ADCChannel::Dial => {
                dial = (adc.read().unwrap() as f32) / MAX_DIAL;
                // TODO: Can we just get the current channel selected on the ADC, rather than
                // having the ADCChannel enum?
                // TODO: Also definitely extract this logic out
                adc.select_channel(channel::SingleA1).unwrap();
                current_channel = ADCChannel::Volume;
            }
            ADCChannel::Volume => {
                volume = (adc.read().unwrap() as f32) / MAX_DIAL;
                adc.select_channel(channel::SingleA0).unwrap();
                current_channel = ADCChannel::Dial;
            }
        }

        manager.tick(dial, volume)?;
    }
}
