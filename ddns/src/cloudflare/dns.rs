use serde::{Deserialize, Serialize};
use worker::*;

#[derive(Debug, Deserialize)]
pub struct ZonesResponse {
    pub result: Option<Vec<Zone>>,
    pub success: bool,
    pub errors: Vec<ErrorMessage>,
}

#[derive(Debug, Deserialize)]
pub struct Zone {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct DnsResponse {
    pub result: Option<Vec<DnsRecord>>,
    pub success: bool,
    pub errors: Vec<ErrorMessage>,
}

#[derive(Debug, Deserialize)]
pub struct DnsRecord {
    pub id: String,
    pub name: String,
    pub r#type: String,
    pub content: String,
    pub proxied: bool,
}

#[derive(Serialize)]
pub struct PatchData {
    pub content: String,
}
#[derive(Debug, Deserialize)]
pub struct UpdateDnsRecordResult {
    pub result: Option<DnsRecord>,
    pub success: bool,
    pub errors: Vec<ErrorMessage>,
}

#[derive(Debug, Deserialize)]
pub struct ErrorMessage {
    pub code: u32,
    pub message: String,
}

trait ErrorMessagesExt {
    fn get_error(&self) -> Error;
}

impl ErrorMessagesExt for Vec<ErrorMessage> {
    fn get_error(&self) -> Error {
        self.first()
            .map(|e| e.message.as_str())
            .unwrap_or("")
            .into()
    }
}
// #[derive(Debug, Deserialize)]
// pub struct DnsRecordResult {
//     pub id: String,
//     pub content: String,
// }

pub struct CloudflareApi {
    headers: Headers,
}

impl CloudflareApi {
    pub fn new(api_token: &str) -> Result<Self> {
        let headers = Headers::new();
        headers.set("Authorization", &format!("Bearer {api_token}"))?;
        headers.set("Content-Type", "application/json")?;

        Ok(Self { headers })
    }
    /*
    List zones
    curl https://api.cloudflare.com/client/v4/zones \
        -H "Authorization: Bearer $CLOUDFLARE_API_TOKEN"
    */
    pub async fn list_zones(&self) -> Result<ZonesResponse> {
        let url = "https://api.cloudflare.com/client/v4/zones";
        let mut init = RequestInit::new();
        init.with_method(Method::Get)
            .with_headers(self.headers.clone());
        let request = Request::new_with_init(url, &init)?;
        let mut response = Fetch::Request(request).send().await?;
        let data: ZonesResponse = response.json().await?;
        if data.success == false {
            return Err(data.errors.get_error());
        }
        Ok(data)
    }
    /*
    List DNS Records
    curl https://api.cloudflare.com/client/v4/zones/$ZONE_ID/dns_records \
        -H "Authorization: Bearer $CLOUDFLARE_API_TOKEN"
    */
    pub async fn list_dns_records(&self, zone_id: &str) -> Result<DnsResponse> {
        let url = format!(
            "https://api.cloudflare.com/client/v4/zones/{}/dns_records",
            zone_id
        );

        let mut init = RequestInit::new();
        init.with_method(Method::Get)
            .with_headers(self.headers.clone());
        let request = Request::new_with_init(&url, &init)?;
        let mut response = Fetch::Request(request).send().await?;
        let data: DnsResponse = response.json().await?;
        if data.success == false {
            return Err(data.errors.get_error());
        }
        Ok(data)
    }
    /*
    Overwrite DNS Record
    curl https://api.cloudflare.com/client/v4/zones/$ZONE_ID/dns_records/$DNS_RECORD_ID \
        -X PUT \
        -H 'Content-Type: application/json' \
        -H "Authorization: Bearer $CLOUDFLARE_API_TOKEN" \
        -d '{
              "name": "example.com",
              "ttl": 3600,
              "type": "A",
              "comment": "Domain verification record",
              "content": "198.51.100.4",
              "private_routing": true,
              "proxied": true
            }'
    */
    pub async fn update_dns_record(
        &self,
        zone_id: &str,
        dns_record_id: &str,
        content: &str,
    ) -> Result<UpdateDnsRecordResult> {
        let url = format!(
            "https://api.cloudflare.com/client/v4/zones/{}/dns_records/{}",
            zone_id, dns_record_id
        );

        let mut init = RequestInit::new();
        init.with_method(Method::Patch)
            .with_headers(self.headers.clone());
        init.with_body(Some(
            serde_json::to_string(&PatchData {
                content: content.into(),
            })?
            .into(),
        ));
        let request = Request::new_with_init(&url, &init)?;
        let mut response = Fetch::Request(request).send().await?;
        let data: UpdateDnsRecordResult = response.json().await?;
        if data.success == false {
            return Err(data.errors.get_error());
        }
        Ok(data)
    }
    /*
    DNS Record Details
    curl https://api.cloudflare.com/client/v4/zones/$ZONE_ID/dns_records/$DNS_RECORD_ID \
    -H "Authorization: Bearer $CLOUDFLARE_API_TOKEN"
    */
    pub async fn get_dns_record_details(
        &self,
        zone_id: &str,
        dns_record_id: &str,
    ) -> Result<UpdateDnsRecordResult> {
        let url = format!(
            "https://api.cloudflare.com/client/v4/zones/{}/dns_records/{}",
            zone_id, dns_record_id
        );
        let mut init = RequestInit::new();
        init.with_method(Method::Get)
            .with_headers(self.headers.clone());

        let request = Request::new_with_init(&url, &init)?;
        let mut response = Fetch::Request(request).send().await?;
        let data: UpdateDnsRecordResult = response.json().await?;
        if data.success == false {
            return Err(data.errors.get_error());
        }
        Ok(data)
    }
}

pub async fn get_zone_id(domain: &str, token: &str) -> Result<String> {
    let api = CloudflareApi::new(token)?;
    let zones = api.list_zones().await?;
    zones
        .result
        .as_ref()
        .and_then(|zones| zones.iter().find(|zone| zone.name == domain))
        .map(|zone| zone.id.clone())
        .ok_or("Zone not found".into())
}

pub async fn get_dns_record_id(
    zone_id: &str,
    full_domain: &str,
    record_type: &str,
    token: &str,
) -> Result<String> {
    let api = CloudflareApi::new(token)?;
    let dns_records = api.list_dns_records(zone_id).await?;
    dns_records
        .result
        .as_ref()
        .and_then(|records| {
            records
                .iter()
                .find(|record| record.name == full_domain && record.r#type == record_type)
        })
        .map(|record| record.id.clone())
        .ok_or("Not Found Record".into())
}

pub async fn update_dns_record(
    zone_id: &str,
    dns_record_id: &str,
    content: &str,
    token: &str,
) -> Result<()> {
    let api = CloudflareApi::new(token)?;
    let _result = api
        .update_dns_record(zone_id, dns_record_id, content)
        .await?;
    Ok(())
}

pub async fn _get_dns_record_details(
    zone_id: &str,
    dns_record_id: &str,
    token: &str,
) -> Result<UpdateDnsRecordResult> {
    let api = CloudflareApi::new(token)?;
    let result = api
        .get_dns_record_details(zone_id, dns_record_id)
        .await?;
    Ok(result)
}
