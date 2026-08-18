use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub kuksa: KuksaConfig,
    pub server: ServerConfig,
    pub subscriptions: Vec<Subscription>,
}

#[derive(Debug, Deserialize)]
pub struct KuksaConfig {
    pub address: String,
}

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub listen: String,
}

#[derive(Debug, Deserialize)]
pub struct Subscription {
    pub path: String,
    pub transform: String,
}

pub fn load(path: &str) -> anyhow::Result<Config> {
    let text = std::fs::read_to_string(path)?;
    let mut cfg: Config = serde_yaml::from_str(&text)?;
    // Allow overriding the KUKSA address without editing the file,
    // e.g. when running inside a container: KUKSA_ADDRESS=http://host.docker.internal:55555
    if let Ok(addr) = std::env::var("KUKSA_ADDRESS") {
        cfg.kuksa.address = addr;
    }
    Ok(cfg)
}
