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

use cache::CachedSignal;
use kuksa::val::v1::{
    datapoint::Value as ProtoValue, val_client::ValClient, Datapoint, Field, SubscribeEntry,
    SubscribeRequest, View,
};
use std::collections::HashMap;
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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cfg = config::load("config.yaml")?;
    let transforms_by_path: HashMap<String, String> = cfg
        .subscriptions
        .iter()
        .map(|s| (s.path.clone(), s.transform.clone()))
        .collect();
    let signals = cache::new_cache();

    println!("Connecting to KUKSA at {}", cfg.kuksa.address);
    let mut client = ValClient::connect(cfg.kuksa.address.clone()).await?;

    let entries: Vec<SubscribeEntry> = cfg
        .subscriptions
        .iter()
        .map(|s| SubscribeEntry {
            path: s.path.clone(),
            view: View::CurrentValue as i32,
            fields: vec![Field::Value as i32],
        })
        .collect();

    println!(
        "Subscribing to {} paths: {:?}",
        entries.len(),
        cfg.subscriptions.iter().map(|s| &s.path).collect::<Vec<_>>()
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

    println!("Stream closed by server");
    Ok(())
}
