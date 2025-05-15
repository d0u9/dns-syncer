use std::collections::HashMap;
use std::process::exit;

use clap::Parser;
use tokio;

use dns_syncer::error::Result;
use dns_syncer::fetcher::Fetcher;
use dns_syncer::fetcher::HttpFetcher;
use dns_syncer::provider::BackendRecords;
use dns_syncer::provider::Cloudflare;
use dns_syncer::provider::Provider;
use dns_syncer::types::FetcherRecordSet;
use dns_syncer::types::ZoneName;

mod config;

type FetcherMap = HashMap<String, Box<dyn Fetcher>>;
type ProviderMap = HashMap<String, Box<dyn Provider>>;

#[derive(Parser)]
struct Args {
    #[clap(short, long)]
    config: String,
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let args = Args::parse();

    let config = config::Parser::parse_yaml(&args.config).unwrap();
    let mut runner = init_runner(config).unwrap();
    runner.run().await.unwrap();

    exit(1);

    // // The key is the provider name, value is the backend records per zone
    // let record_per_provider = to_backend_records(records).unwrap();
    // //dbg!(&record_per_provider);

    // let providers = providers
    //     .into_iter()
    //     .filter_map(|p| {
    //         if record_per_provider.contains_key(&p.name) {
    //             Some((p.name.clone(), instance_provider(p).unwrap()))
    //         } else {
    //             None
    //         }
    //     })
    //     .collect::<HashMap<_, _>>();

    // dbg!(
    //     &providers
    //         .iter()
    //         .map(|(name, _)| name.clone())
    //         .collect::<Vec<_>>()
    // );

    // let global_records_clone = global_records.clone();
    // for (provider_name, records) in record_per_provider.iter() {
    //     println!("provider_name: {}", provider_name);
    //     let provider = providers.get(provider_name).unwrap();
    //     provider
    //         .sync(records.clone(), global_records_clone.clone().into())
    //         .await
    //         .unwrap();
    // }
}

#[derive(Debug, Clone, Default)]
struct ProviderBackend {
    record: BackendRecords,
    fetchers: Vec<String>,
}

struct Runner {
    global_fetcher_name: String,
    fetchers: FetcherMap,
    providers: ProviderMap,
    record_per_provider: HashMap<String, ProviderBackend>,
}

fn init_runner(config: config::Cfg) -> Result<Runner> {
    let config::Cfg {
        check_interval: _,
        providers,
        fetchers,
        records,
        public_ip_fecher,
    } = config;

    let fetchers = create_fetchers(&records, &public_ip_fecher, &fetchers).unwrap();
    let providers = create_providers(&records, &providers).unwrap();

    // The key is the provider name, value is the backend records per zone
    let record_per_provider = to_provider_backends(records).unwrap();

    Ok(Runner {
        global_fetcher_name: public_ip_fecher.to_string(),
        fetchers,
        providers,
        record_per_provider,
    })
}

impl Runner {
    async fn run(&mut self) -> Result<()> {
        let public_ip = self.fetch_public_ip().await?;

        for (provider_name, backend) in self.record_per_provider.iter() {
            let provider = self.providers.get_mut(provider_name).unwrap();
            provider
                .sync(backend.record.clone(), public_ip.clone().into())
                .await?;
        }

        Ok(())
    }

    async fn fetch_public_ip(&mut self) -> Result<FetcherRecordSet> {
        let fetcher = self.fetchers.get_mut(&self.global_fetcher_name).unwrap();
        fetcher.fetch().await
    }
}

fn list_in_use_providers(records: &Vec<config::CfgRecordItem>) -> Vec<String> {
    let mut ret = records
        .iter()
        .flat_map(|r| r.providers.iter().map(|p| p.name.clone()))
        .collect::<Vec<_>>();
    ret.sort();
    ret.dedup();
    ret
}

fn create_providers(
    records: &Vec<config::CfgRecordItem>,
    providers: &Vec<config::CfgProvider>,
) -> Result<ProviderMap> {
    let in_use_providers = list_in_use_providers(records);

    let ret = providers
        .into_iter()
        .filter(|f| in_use_providers.contains(&f.name))
        .filter_map(|provider| {
            match provider.r#type.as_str() {
                // Create new Cloudflare provider if authentication is valid
                "cloudflare" => {
                    let auth = provider.authentication.clone().try_into().ok()?;
                    let cloudflare = Cloudflare::new(auth);

                    Some((
                        provider.name.clone(),
                        Box::new(cloudflare) as Box<dyn Provider>,
                    ))
                }
                // Skip unknown provider types
                _ => None,
            }
        })
        .collect::<HashMap<_, _>>();
    Ok(ret)
}

fn list_in_use_fethers(
    records: &Vec<config::CfgRecordItem>,
    public_ip_fecher: &str,
) -> Vec<String> {
    let mut ret = records
        .iter()
        .flat_map(|r| r.fetchers.iter().map(|f| f.name.clone()))
        .collect::<Vec<_>>();
    ret.push(public_ip_fecher.to_string());
    ret.sort();
    ret.dedup();
    ret
}

fn create_fetchers(
    records: &Vec<config::CfgRecordItem>,
    public_ip_fecher: &str,
    fetchers: &Vec<config::CfgFetcher>,
) -> Result<FetcherMap> {
    let in_use_fetchers = list_in_use_fethers(records, public_ip_fecher);

    let ret = fetchers
        .into_iter()
        .filter(|f| in_use_fetchers.contains(&f.name))
        .filter_map(|fetcher| {
            // Create appropriate fetcher based on type
            match fetcher.r#type.as_str() {
                // For HTTP fetchers, create new instance with params
                "http_fetcher" => {
                    let http_fetcher = HttpFetcher::new_with_args(fetcher.params.clone().into());
                    Some((
                        fetcher.name.clone(),
                        Box::new(http_fetcher) as Box<dyn Fetcher>,
                    ))
                }
                // Skip any unknown fetcher types
                _ => None,
            }
        })
        .collect::<HashMap<_, _>>();
    Ok(ret)
}

fn to_provider_backends(
    cfg_records: Vec<config::CfgRecordItem>,
) -> Result<HashMap<String, ProviderBackend>> {
    let mut ret: HashMap<String, ProviderBackend> = HashMap::new();

    for item in cfg_records {
        process_record_item(&mut ret, item);
    }
    Ok(ret)
}

fn process_record_item(
    records_map: &mut HashMap<String, ProviderBackend>,
    item: config::CfgRecordItem,
) {
    for provider in item.providers {
        process_provider(records_map, &item.record, provider);
    }
}

fn process_provider(
    records_map: &mut HashMap<String, ProviderBackend>,
    record: &config::CfgRecord,
    provider: config::CfgRecordProvider,
) {
    let provider_name = provider.name;
    let backend_records = records_map
        .entry(provider_name)
        .or_insert(Default::default());

    for zone in provider.zones {
        add_zone_record(&mut backend_records.record, zone, record, &provider.params);
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
