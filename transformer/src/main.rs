mod api;
mod cache;
mod config;
mod transforms;

pub mod kuksa {
    pub mod val {
        pub mod v1 {
            tonic::include_proto!("kuksa.val.v1");
        }
    }
}

use cache::{CachedSignal, SignalCache};
use kuksa::val::v1::{
    datapoint::Value as ProtoValue, val_client::ValClient, Datapoint, Field, SubscribeEntry,
    SubscribeRequest, View,
};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;
use transforms::SignalValue;

fn extract(dp: &Datapoint) -> Option<SignalValue> {
    match dp.value.as_ref()? {
        ProtoValue::Float(v) => Some(SignalValue::Float(*v as f64)),
        ProtoValue::Double(v) => Some(SignalValue::Float(*v)),
        ProtoValue::Bool(b) => Some(SignalValue::Bool(*b)),
        ProtoValue::Int32(v) => Some(SignalValue::Float(*v as f64)),
        ProtoValue::Int64(v) => Some(SignalValue::Float(*v as f64)),
        ProtoValue::Uint32(v) => Some(SignalValue::Float(*v as f64)),
        ProtoValue::Uint64(v) => Some(SignalValue::Float(*v as f64)),
        ProtoValue::String(s) => Some(SignalValue::Text(s.clone())),
        _ => None, // arrays not needed for the configured paths
    }
}

/// One subscribe session: connect, subscribe, consume the stream until it ends.
async fn subscribe_once(
    address: &str,
    subscriptions: &[config::Subscription],
    transforms_by_path: &HashMap<String, String>,
    signals: &SignalCache,
) -> anyhow::Result<()> {
    let mut client = ValClient::connect(address.to_string()).await?;

    let entries: Vec<SubscribeEntry> = subscriptions
        .iter()
        .map(|s| SubscribeEntry {
            path: s.path.clone(),
            view: View::CurrentValue as i32,
            fields: vec![Field::Value as i32],
        })
        .collect();

    println!(
        "Subscribed to {} paths: {:?}",
        entries.len(),
        subscriptions.iter().map(|s| &s.path).collect::<Vec<_>>()
    );

    let mut stream = client
        .subscribe(SubscribeRequest { entries })
        .await?
        .into_inner();

    while let Some(response) = stream.message().await? {
        for update in response.updates {
            let Some(entry) = update.entry else { continue };
            let Some(raw) = entry.value.as_ref().and_then(extract) else {
                continue; // no value yet for this path
            };
            let transform = transforms_by_path
                .get(&entry.path)
                .cloned()
                .unwrap_or_else(|| "passthrough".to_string());
            let cooked = transforms::apply(&transform, raw.clone());
            println!("{}: {:?} -[{}]-> {:?}", entry.path, raw, transform, cooked);
            signals.write().unwrap().insert(
                entry.path.clone(),
                CachedSignal {
                    path: entry.path.clone(),
                    value: cooked,
                    transform,
                    updated_at: chrono::Utc::now().to_rfc3339(),
                },
            );
        }
    }
    Ok(())
}

/// Keep the subscription alive forever, reconnecting with exponential backoff.
/// The cache keeps serving the last known values while disconnected.
async fn run_subscriber(cfg: config::Config, signals: SignalCache) {
    let transforms_by_path: HashMap<String, String> = cfg
        .subscriptions
        .iter()
        .map(|s| (s.path.clone(), s.transform.clone()))
        .collect();

    let mut backoff = Duration::from_secs(1);
    loop {
        println!("Connecting to KUKSA at {}", cfg.kuksa.address);
        match subscribe_once(&cfg.kuksa.address, &cfg.subscriptions, &transforms_by_path, &signals)
            .await
        {
            Ok(()) => println!("Stream closed by server, reconnecting"),
            Err(e) => println!("Subscription error: {e}, retrying in {backoff:?}"),
        }
        tokio::time::sleep(backoff).await;
        backoff = (backoff * 2).min(Duration::from_secs(30));
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cfg = config::load("config.yaml")?;
    let signals = cache::new_cache();

    let state = api::ApiState {
        signals: signals.clone(),
        configured_paths: Arc::new(
            cfg.subscriptions
                .iter()
                .map(|s| s.path.clone())
                .collect::<HashSet<_>>(),
        ),
    };
    let listen = cfg.server.listen.clone();

    tokio::spawn(run_subscriber(cfg, signals));

    println!("Serving REST API on {listen}");
    let listener = tokio::net::TcpListener::bind(&listen).await?;
    axum::serve(listener, api::router(state)).await?;
    Ok(())
}
