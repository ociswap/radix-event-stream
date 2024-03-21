use core::panic;
use scrypto::network::NetworkDefinition;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct SettingsParsed {
    pub network: String,
    pub radix_gateway_url: String,
    pub start_from_state_version: u64,
    pub limit_per_page: u32,
    pub log_level: String,
}

#[derive(Debug)]
pub struct Settings {
    pub network: NetworkDefinition,
    pub radix_gateway_url: String,
    pub start_from_state_version: u64,
    pub limit_per_page: u32,
    pub log_level: log::Level,
}

impl Settings {
    pub fn init() -> Settings {
        dotenv::dotenv().ok();

        let builder = config::Config::builder()
            .add_source(config::Environment::default())
            .build()
            .expect("Failed to build config");

        let parsed: SettingsParsed = builder
            .try_deserialize()
            .expect("The .env should match the ConfigParsed struct");

        let network = match parsed.network.as_str() {
            "mainnet" => NetworkDefinition::mainnet(),
            "stokenet" => NetworkDefinition::stokenet(),
            _ => panic!("'{}' is not a valid network.", parsed.network),
        };

        let log_level = match parsed.log_level.as_str() {
            "trace" => log::Level::Trace,
            "debug" => log::Level::Debug,
            "info" => log::Level::Info,
            "warn" => log::Level::Warn,
            "error" => log::Level::Error,
            _ => log::Level::Info,
        };

        Settings {
            network: network,
            radix_gateway_url: parsed.radix_gateway_url,
            start_from_state_version: parsed.start_from_state_version,
            limit_per_page: parsed.limit_per_page,
            log_level: log_level,
        }
    }
}
