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
    dbg!(&r);
}

fn to_backend_records(
    cfg_records: Vec<config::CfgRecordItem>,
) -> Result<HashMap<ZoneName, BackendRecords>> {
    let mut ret: HashMap<ZoneName, BackendRecords> = HashMap::new();

    for item in cfg_records {
        process_record_item(&mut ret, item);
    }
    Ok(ret)
}

fn process_record_item(
    records_map: &mut HashMap<ZoneName, BackendRecords>,
    item: config::CfgRecordItem,
) {
    for backend in item.backends {
        process_backend(records_map, &item.record, backend);
    }
}

fn process_backend(
    records_map: &mut HashMap<ZoneName, BackendRecords>,
    record: &config::CfgRecord,
    backend: config::CfgRecordBackend,
) {
    let provider_name = backend.provider;
    let backend_records = records_map
        .entry(provider_name)
        .or_insert(Default::default());

    for zone in backend.zones {
        add_zone_record(backend_records, zone, record, &backend.params);
    }
}

fn add_zone_record(
    backend_records: &mut BackendRecords,
    zone: ZoneName,
    record: &config::CfgRecord,
    params: &Vec<config::CfgRecordBackendParams>,
) {
    let zone_records = backend_records
        .zones
        .entry(zone)
        .or_insert(Default::default());
    let provider_record = record.clone().into_provider_record(params);
    zone_records.records.push(provider_record);
}
