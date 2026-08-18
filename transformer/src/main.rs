mod config;

pub mod kuksa {
    pub mod val {
        pub mod v1 {
            tonic::include_proto!("kuksa.val.v1");
        }
    }
}

use kuksa::val::v1::{val_client::ValClient, Field, SubscribeEntry, SubscribeRequest, View};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cfg = config::load("config.yaml")?;
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
            if let Some(entry) = update.entry {
                println!("update: {} = {:?}", entry.path, entry.value);
            }
        }
    }

    println!("Stream closed by server");
    Ok(())
}
