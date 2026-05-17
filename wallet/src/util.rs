use crate::core::{Config, Core, FeeConfig, FeeType, Recipient};
use anyhow::Result;
use std::{panic, path::PathBuf};
use tracing::*;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{EnvFilter, prelude::*};

pub fn setup_tracing() -> Result<()> {
    let _file_appender = RollingFileAppender::new(Rotation::DAILY, "logs", "wallet.log");

    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env().add_directive(tracing::Level::TRACE.into()))
        .init();

    Ok(())
}

pub fn setup_panic_hook() {
    panic::set_hook(Box::new(|panic_info| {
        let backtrace = std::backtrace::Backtrace::force_capture();
        error!("Application panicked!");
        error!("Panic info: {:?}", panic_info);
        error!("Backtrace: {:?}", backtrace);
    }));
}

pub fn generate_dummy_config(path: &PathBuf) -> Result<()> {
    let dummy_config = Config {
        my_keys: vec![],
        contacts: vec![
            Recipient {
                name: String::from("Samuel"),
                key: PathBuf::from("sam.pub.pem"),
            },
            Recipient {
                name: String::from("Alice"),
                key: PathBuf::from("alice.pub.pem"),
            },
        ],
        default_node: String::from("127.0.0.1:9000"),
        fee_config: FeeConfig {
            fee_type: FeeType::Percent,
            value: 0.1,
        },
    };

    let config_str = toml::to_string_pretty(&dummy_config)?;
    std::fs::write(path, config_str)?;

    println!("Dummy config generated at: {}", path.display());
    Ok(())
}

pub fn sats_to_btc(sats: u64) -> String {
    format!("{:.2} BTC", sats as f64 / 100_000_000.0)
}

pub fn big_mode_btc(core: &Core) -> String {
    text_to_ascii_art::to_art(sats_to_btc(core.get_balance()), "default", 0, 0, 0).unwrap()
}
