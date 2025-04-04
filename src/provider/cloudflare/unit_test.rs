use std::net::Ipv4Addr;

use super::cloudflare::*;
use crate::record::ProviderParam;
use crate::record::ProviderRecord;
use crate::record::RecordContent;
use crate::record::RecordOp;
use crate::record::TTL;

#[tokio::test]
async fn test_cf_record_op_purge() {
    let (zone_name, zone_id) = zone_name();
    let record = ProviderRecord {
        name: format!("testcf.{}", zone_name),
        content: RecordContent::A(Ipv4Addr::new(1, 2, 3, 5)),
        comment: Some("unit test test_cf_record_op_create".to_string()),
        ttl: TTL::Value(3600),
        op: RecordOp::Create,
        params: vec![ProviderParam {
            name: "proxied".to_string(),
            value: "true".to_string(),
        }],
    };

    let cli = init_cli();
    let _resp = cli.record_op_purge(&zone_id, record).await.unwrap();
}

#[tokio::test]
async fn test_cf_record_op_create() {
    let (zone_name, zone_id) = zone_name();
    let record = ProviderRecord {
        name: format!("testcf.{}", zone_name),
        content: RecordContent::A(Ipv4Addr::new(1, 2, 3, 5)),
        comment: Some("unit test test_cf_record_op_create".to_string()),
        ttl: TTL::Auto,
        op: RecordOp::Create,
        params: vec![ProviderParam {
            name: "proxied".to_string(),
            value: "true".to_string(),
        }],
    };

    let cli = init_cli();
    println!("{:?}", CfRecord::from(record.clone()));
    let _resp = cli.record_op_create(&zone_id, record).await.unwrap();
}

#[tokio::test]
async fn test_cf_records_list_by_name() {
    let cli = init_cli();
    let (zone_name, zone_id) = zone_name();
    let name = format!("testcf.{}", zone_name);
    let records = cli.records_list_by_name(&zone_id, &name).await.unwrap();
    println!("{:?}", records);
}

#[tokio::test]
async fn test_cf_records_list() {
    let cli = init_cli();
    let (_name, id) = zone_name();
    let records = cli.records_list(&id).await.unwrap();
    println!("{:?}", records);
}

#[tokio::test]
async fn test_cf_zone_list() {
    let cli = init_cli();
    let (name, id) = zone_name();
    let zone = cli.zone_list(&name).await.unwrap();
    println!("{:?}", zone);
    if let Some(zone) = zone {
        assert_eq!(zone.id, id);
    } else {
        panic!("Zone not found");
    }
}

#[test]
fn test_cf_record_deserialize() {
    let json = r#"{
        "comment": "hello",
        "content": "42.192.202.2",
        "created_on": "2022-06-08T02:19:45.956932Z",
        "id": "79de548c4af681c2af1a9e92be42d004",
        "meta": {},
        "modified_on": "2022-06-08T02:19:45.956932Z",
        "name": "cn.d0u9.top",
        "proxiable": true,
        "proxied": true,
        "settings": {},
        "tags": [],
        "ttl": 1,
        "type": "A"
    }"#;
    let record: CfRecord = serde_json::from_str(json).unwrap();
    assert_eq!(record.comment, Some("hello".to_string()));
    assert_eq!(
        record.content,
        RecordContent::A(Ipv4Addr::new(42, 192, 202, 2))
    );
    assert_eq!(record.proxied, true);
    println!("{:?}", record);
}

#[test]
fn test_cf_auth_deserialize() {
    let yaml = r#"
type: api_token
value: "1234567890"
        "#;
    let auth: Auth = serde_yaml::from_str(yaml).unwrap();
    if let Auth::ApiToken(token) = auth {
        assert_eq!(token, "1234567890");
    } else {
        panic!("Expected ApiToken");
    }

    let yaml = r#"
type: api_key
value:
  email: "test@example.com"
  key: "1234567890"
        "#;
    let auth: Auth = serde_yaml::from_str(yaml).unwrap();
    if let Auth::ApiKey { email, key } = auth {
        assert_eq!(email, "test@example.com");
        assert_eq!(key, "1234567890");
    } else {
        panic!("Expected ApiKey");
    }
}

fn init_cli() -> Cli {
    let token = std::env::var("CF_API_TOKEN").unwrap();
    let auth = Auth::ApiToken(token);
    Cli::new(auth)
}

fn zone_name() -> (String, String) {
    let name = std::env::var("CF_ZONE_NAME").unwrap();
    let id = std::env::var("CF_ZONE_ID").unwrap();
    (name, id)
}
