use std::collections::HashMap;

use clap::Parser;
use tokio;

use dns_syncer::error::Result;
use dns_syncer::provider::BackendRecords;
use dns_syncer::record::ZoneName;

mod config;

#[derive(Parser)]
struct Args {
    #[clap(short, long)]
    config: String,
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let args = Args::parse();

    let config = config::Parser::parse_yaml(&args.config).unwrap();
    let config::Cfg {
        check_interval,
        providers,
        fetchers,
        records,
    } = config;
    println!("config: {:?}", records);
    let r = to_backend_records(records);
}

fn to_backend_records(
    cfg_records: Vec<config::CfgRecordItem>,
) -> Result<HashMap<ZoneName, BackendRecords>> {
    let mut ret: HashMap<ZoneName, BackendRecords> = HashMap::new();

    for item in cfg_records {
        for backend in item.backends {
            let provider_name = backend.provider;
            let backend_records = ret.entry(provider_name).or_insert(Default::default());
            for zone in backend.zones {
                let zone_records = backend_records
                    .zones
                    .entry(zone)
                    .or_insert(Default::default());
                //zone_records.records.push(item.record);
            }
        }
    }
    Ok(ret)
}
