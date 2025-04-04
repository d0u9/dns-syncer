use std::sync::Mutex;
use std::time;

use crate::error::{Error, Result};
use crate::fetcher::Fetcher;
use crate::fetcher::HttpFetcher;
use crate::record::RecordEntrySet;

static GLOBAL_FETCHER: Mutex<Option<GlobalFetcher>> = Mutex::new(None);

struct GlobalFetcher {
    lifetime: time::Duration,
    cache_result: RecordEntrySet,
    last_fetch_time: time::Instant,
    fetcher: Box<dyn Fetcher + Send + Sync>,
}

impl Default for GlobalFetcher {
    fn default() -> Self {
        Self {
            lifetime: time::Duration::from_secs(20), // 5 minutes default
            cache_result: RecordEntrySet::default(),
            last_fetch_time: time::Instant::now(),
            fetcher: Box::new(HttpFetcher::default()),
        }
    }
}

impl GlobalFetcher {
    fn is_fresh(&self) -> bool {
        self.last_fetch_time.elapsed() < self.lifetime && !self.cache_result.is_empty()
    }
}

pub async fn fetch() -> Result<RecordEntrySet> {
    let mut guard = GLOBAL_FETCHER
        .lock()
        .map_err(|_| Error::GlobalFetcherError("failed to lock global fetcher".to_string()))?;

    let fetcher = guard.get_or_insert_with(|| GlobalFetcher::default());

    if fetcher.is_fresh() {
        return Ok(fetcher.cache_result.clone());
    }

    let records = fetcher.fetcher.fetch().await?;
    fetcher.cache_result = records.clone();
    fetcher.last_fetch_time = time::Instant::now();
    Ok(records)
}

pub fn set_global_fetcher(cfg: GlobalFetcherCfg) -> Result<()> {
    let mut guard = GLOBAL_FETCHER
        .lock()
        .map_err(|_| Error::GlobalFetcherError("failed to lock global fetcher".to_string()))?;

    let mut global = GlobalFetcher::default();
    if let Some(lifetime) = cfg.lifetime {
        global.lifetime = lifetime;
    }

    if let Some(fetcher) = cfg.fetcher {
        global.fetcher = fetcher;
    }

    *guard = Some(global);
    Ok(())
}

pub struct GlobalFetcherCfg {
    lifetime: Option<time::Duration>,
    fetcher: Option<Box<dyn Fetcher + Send + Sync>>,
}

impl Default for GlobalFetcherCfg {
    fn default() -> Self {
        Self::new()
    }
}

impl GlobalFetcherCfg {
    fn new() -> Self {
        Self {
            lifetime: None,
            fetcher: None,
        }
    }

    #[allow(dead_code)]
    fn set_lifetime(mut self, lifetime: time::Duration) -> Self {
        self.lifetime = Some(lifetime);
        self
    }

    #[allow(dead_code)]
    fn set_fetcher(mut self, fetcher: Box<dyn Fetcher + Send + Sync>) -> Self {
        self.fetcher = Some(fetcher);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_global_fetcher() {
        let records = fetch().await.unwrap();
        assert!(!records.is_empty());
        println!("records: {:?}", records);
    }
}
