use ads1x1x::{Ads1x1x, DataRate16Bit, FullScaleRange, TargetAddr, channel};
use anyhow::Result;
use linux_embedded_hal::I2cdev;
use std::{
    sync::{
        Arc,
        atomic::{AtomicI16, Ordering},
    },
    thread,
    time::Duration,
};

// Since the scale range is for ~4.096V and we use 3.3V as reference, we need to target ~26400 as
// the max value we'll read from the potentiometer;
const MAX_DIAL: f32 = (3300.0 / 4096.0) * 32768.0;

pub struct ADCReader {
    dial: Arc<AtomicI16>,
    volume: Arc<AtomicI16>,
}

impl ADCReader {
    pub fn spawn() -> Result<Self> {
        let dial = Arc::new(AtomicI16::new(0));
        let volume = Arc::new(AtomicI16::new(0));

        thread::spawn({
            let dial = Arc::clone(&dial);
            let volume = Arc::clone(&volume);

            move || {
                let dev = I2cdev::new("/dev/i2c-1").unwrap();
                let mut adc = Ads1x1x::new_ads1115(dev, TargetAddr::default());
                adc.set_full_scale_range(FullScaleRange::Within4_096V)
                    .unwrap();
                adc.set_data_rate(DataRate16Bit::Sps475).unwrap();
                let mut adc = adc
                    .into_continuous()
                    .map_err(|_| anyhow::anyhow!("failed to enter continuous mode"))
                    .unwrap();

                adc.select_channel(channel::SingleA0).unwrap();
                loop {
                    adc.select_channel(channel::SingleA0).unwrap();
                    thread::sleep(Duration::from_millis(5));
                    let raw = adc.read().unwrap();
                    dial.store(raw, Ordering::Relaxed);

                    adc.select_channel(channel::SingleA1).unwrap();
                    thread::sleep(Duration::from_millis(5));
                    let raw = adc.read().unwrap();
                    volume.store(raw, Ordering::Relaxed);
                }
            }
        });

        Ok(Self { dial, volume })
    }

    pub fn dial(&self) -> f32 {
        let dial = self.dial.load(Ordering::Relaxed);
        dial as f32 / MAX_DIAL
    }
    pub fn volume(&self) -> f32 {
        let volume = self.volume.load(Ordering::Relaxed);
        volume as f32 / MAX_DIAL
    }
}
