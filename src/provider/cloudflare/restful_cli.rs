use crate::error::Result;

use super::cloudflare::{CfRecord, CfRecordOp};

pub(super) struct CfClient {
    api_token: String,
}

impl CfClient {
    pub fn new(api_token: String) -> Self {
        Self { api_token }
    }

    pub async fn sync(&self, zone: &str, records: &[CfRecord]) -> Result<()> {
        Ok(())
    }
}

struct ReqBody {
    body: String,
}

impl From<Vec<CfRecord>> for ReqBody {
    fn from(records: Vec<CfRecord>) -> Self {
        Self {
            body: String::from(""),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_req_body_from_records() {
        let records = vec![CfRecord {
            r#type: "A".to_string(),
            name: "example.com".to_string(),
            content: Some("192.168.1.1".to_string()),
            proxied: false,
            comment: "".to_string(),
            op: CfRecordOp::Overwrite,
        }];

        let req_body = ReqBody::from(records);
        println!("req_body: {}", req_body.body);
    }
}
