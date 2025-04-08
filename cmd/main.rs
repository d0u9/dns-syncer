use std::collections::HashMap;

use clap::Parser;
use tokio;

use dns_syncer::error::Error;
use dns_syncer::error::Result;
use dns_syncer::fetcher::HttpFetcher;
use dns_syncer::provider::BackendRecords;
use dns_syncer::provider::Cloudflare;
use dns_syncer::provider::Provider;
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
        check_interval: _,
        providers,
        fetchers: _,
        records,
    } = config;

    let global_fetcher = HttpFetcher::default();
    let global_records = global_fetcher.fetch().await.unwrap();
    println!("fetched records: {:?}", global_records);

    println!("config: {:?}", records);
    // The key is the provider name, value is the backend records per zone
    let record_per_provider = to_backend_records(records).unwrap();
    //dbg!(&record_per_provider);

    let providers = providers
        .into_iter()
        .filter_map(|p| {
            if record_per_provider.contains_key(&p.name) {
                Some((p.name.clone(), instance_provider(p).unwrap()))
            } else {
                None
            }
        })
        .collect::<HashMap<_, _>>();

    dbg!(
        &providers
            .iter()
            .map(|(name, _)| name.clone())
            .collect::<Vec<_>>()
    );

    let global_records_clone = global_records.clone();
    for (provider_name, records) in record_per_provider.iter() {
        println!("provider_name: {}", provider_name);
        let provider = providers.get(provider_name).unwrap();
        provider
            .sync(records.clone(), global_records_clone.clone().into())
            .await
            .unwrap();
    }
}

fn instance_provider(provider: config::CfgProvider) -> Result<Box<dyn Provider>> {
    let provider = match provider.r#type.as_str() {
        "cloudflare" => Box::new(Cloudflare::new(
            provider.name,
            provider.authentication.try_into()?,
        )),
        _ => return Err(Error::Provider(provider.r#type)),
    };
    Ok(provider)
}

fn to_backend_records(
    cfg_records: Vec<config::CfgRecordItem>,
) -> Result<HashMap<String, BackendRecords>> {
    let mut ret: HashMap<String, BackendRecords> = HashMap::new();

    for item in cfg_records {
        process_record_item(&mut ret, item);
    }
    Ok(ret)
}

fn process_record_item(
    records_map: &mut HashMap<String, BackendRecords>,
    item: config::CfgRecordItem,
) {
    for provider in item.providers {
        process_provider(records_map, &item.record, provider);
    }
}

fn process_provider(
    records_map: &mut HashMap<String, BackendRecords>,
    record: &config::CfgRecord,
    provider: config::CfgRecordProvider,
) {
    let provider_name = provider.name;
    let backend_records = records_map
        .entry(provider_name)
        .or_insert(Default::default());

    for zone in provider.zones {
        add_zone_record(backend_records, zone, record, &provider.params);
    }
}

fn add_zone_record(
    backend_records: &mut BackendRecords,
    zone: ZoneName,
    record: &config::CfgRecord,
    params: &config::CfgParamList,
) {
    let zone_records = backend_records
        .zones
        .entry(zone)
        .or_insert(Default::default());
    let provider_record = record.clone().into_provider_record(params);
    zone_records.records.push(provider_record);
}
