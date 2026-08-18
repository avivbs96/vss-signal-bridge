use clap::Parser;

/// VssDataClient: query the VssDataTransformer for a signal value.
#[derive(Parser)]
#[command(name = "vss-client")]
struct Args {
    /// Base address of the transformer, e.g. http://localhost:8080
    #[arg(long, default_value = "http://localhost:8080")]
    addr: String,

    /// VSS path to query, e.g. Vehicle.Speed
    #[arg(long)]
    path: String,

    /// Print the full JSON record instead of just the value
    #[arg(long)]
    full: bool,
}

fn main() {
    let args = Args::parse();
    let url = format!("{}/signals/{}", args.addr.trim_end_matches('/'), args.path);

    match ureq::get(&url).call() {
        Ok(resp) => {
            let body: serde_json::Value = resp
                .into_json()
                .unwrap_or(serde_json::Value::Null);
            if args.full {
                println!("{}", body);
            } else {
                match body.get("value") {
                    Some(v) => println!("{}", v),
                    None => println!("{}", body),
                }
            }
        }
        Err(ureq::Error::Status(code, resp)) => {
            let body = resp.into_string().unwrap_or_default();
            eprintln!("HTTP {code}: {body}");
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("request failed: {e}");
            std::process::exit(1);
        }
    }
}
