mod driver;
mod font;
mod protocol;
mod renderer;

use crate::driver::DedDriver;
use crate::renderer::Renderer;
use anyhow::{Context, Result};
use bms_sm::{FlightData, MemoryFile};
use env_logger::Env;
use figment::{
    Figment,
    providers::{Format, Serialized, Toml},
};
use log::{debug, info, warn};
use seahash::SeaHasher;
use serde::{Deserialize, Serialize};
use std::hash::Hasher;
use std::thread;
use std::time::{Duration, Instant};

use crate::protocol::{ICP_USB_PRODUCT_ID, ICP_USB_VENDOR_ID};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub log_level: String,
    pub vendor_id: u16,
    pub product_id: u16,
    pub update_period_ms: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            log_level: "info".to_string(),
            vendor_id: ICP_USB_VENDOR_ID,
            product_id: ICP_USB_PRODUCT_ID,
            update_period_ms: 33,
        }
    }
}

fn main() -> Result<()> {
    let config: Config = Figment::new()
        .merge(Serialized::defaults(Config::default()))
        .merge(Toml::file("config.toml"))
        .extract()
        .context("Failed to parse config.")?;

    let env = Env::default().filter_or("LOG_LEVEL", &config.log_level);
    env_logger::init_from_env(env);

    let version = option_env!("VERGEN_GIT_DESCRIBE").unwrap_or("Could not determine version!");

    info!("falcon-bms-ded-driver: {}", version);
    debug!("Config is: {:?}", &config);

    let renderer = Renderer::new();
    let mut driver_opt: Option<DedDriver> = None;
    let mut last_driver_attempt = Instant::now() - Duration::from_secs(6);

    info!("Waiting for BMS shared memory...");

    // We try to open the memory area. If it fails, we keep retrying (with "Waiting" message).
    let mut last_hash = 0u64;
    let mut last_falcon_running = false;
    let mut waiting_invert = false;
    let mut last_waiting_tick = Instant::now();

    let mut waiting_lines = [[b' '; 26]; 5];
    let msg = b"  \x02WAITING FOR BMS...\x02  ";
    waiting_lines[2][..msg.len()].copy_from_slice(msg);

    let mut waiting_invert_lines = [[b' '; 26]; 5];
    let inv_msg = b"  X                  X  ";
    waiting_invert_lines[2][..inv_msg.len()].copy_from_slice(inv_msg);

    let blank_lines = [[b' '; 26]; 5];

    let mut reader_opt: Option<MemoryFile<FlightData>> = None;

    loop {
        let start_time = Instant::now();

        if driver_opt.is_none() && last_driver_attempt.elapsed() >= Duration::from_secs(5) {
            last_driver_attempt = Instant::now();
            match DedDriver::new(config.vendor_id, config.product_id) {
                Ok(d) => {
                    info!("HID device connected!");
                    driver_opt = Some(d);
                    last_hash = 0;
                }
                Err(e) => {
                    debug!("Failed to find HID device: {}", e);
                }
            }
        }

        if reader_opt.is_none() {
            reader_opt = unsafe { MemoryFile::new("FalconSharedMemoryArea").ok() };
        }

        if let Some(ref reader) = reader_opt {
            let data = reader.read();

            // If Falcon closes, the mapping might still be there but data stays stale.
            // Some BMS drivers check version_num or some bits to see if it's active.
            let falcon_running = data.version_num > 0;

            if !falcon_running {
                if last_falcon_running {
                    debug!("Falcon BMS in UI or stopped. Waiting...");
                    last_falcon_running = false;
                }

                if last_waiting_tick.elapsed() >= Duration::from_secs(1) {
                    waiting_invert = !waiting_invert;
                    last_waiting_tick = Instant::now();

                    let inv = if waiting_invert {
                        &waiting_invert_lines
                    } else {
                        &blank_lines
                    };
                    let rendered = renderer.render(&waiting_lines, inv);

                    if let Some(ref mut driver) = driver_opt {
                        if let Err(e) = driver.send_display_data(&rendered) {
                            warn!("Failed to send data to HID device: {}. Disconnecting.", e);
                            driver_opt = None;
                        }
                    }
                }
            } else {
                if !last_falcon_running {
                    info!("Falcon BMS started!");
                    last_falcon_running = true;
                }

                let ded_lines: &[[u8; 26]; 5] = unsafe { std::mem::transmute(&data.dedlines) };
                let invert_lines: &[[u8; 26]; 5] = unsafe { std::mem::transmute(&data.invert) };

                let rendered = renderer.render(ded_lines, invert_lines);

                let mut hasher = SeaHasher::new();
                hasher.write(&rendered);
                let hash = hasher.finish();

                if hash != last_hash {
                    if let Some(ref mut driver) = driver_opt {
                        match driver.send_display_data(&rendered) {
                            Ok(_) => last_hash = hash,
                            Err(e) => {
                                warn!("Failed to send data to HID device: {}. Disconnecting.", e);
                                driver_opt = None;
                            }
                        }
                    }
                }
            }
        } else {
            // No reader, display waiting message
            if last_waiting_tick.elapsed() >= Duration::from_secs(1) {
                waiting_invert = !waiting_invert;
                last_waiting_tick = Instant::now();

                let inv = if waiting_invert {
                    &waiting_invert_lines
                } else {
                    &blank_lines
                };
                let rendered = renderer.render(&waiting_lines, inv);

                if let Some(ref mut driver) = driver_opt {
                    if let Err(e) = driver.send_display_data(&rendered) {
                        warn!("Failed to send data to HID device: {}. Disconnecting.", e);
                        driver_opt = None;
                    }
                }
            }
        }

        let elapsed = start_time.elapsed();
        let period = Duration::from_millis(config.update_period_ms);
        if elapsed < period {
            thread::sleep(period - elapsed);
        }
    }
}
