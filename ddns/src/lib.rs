mod auth;
mod cloudflare;
mod signature;
use cloudflare::d1::get_logs;
use serde::Deserialize;
use std::future::Future;
use worker::*;

const ZONES_KV: &str = "zones";
const RECORDS_KV: &str = "records";

use time::serde::format_description;
use time::PrimitiveDateTime;

format_description!(
    my_custom_date_format,
    PrimitiveDateTime,
    "[year]-[month]-[day] [hour]:[minute]:[second]"
);

#[derive(Deserialize)]
struct UpdateDnsRequestData {
    zone: String,   // main/base domain
    record: String, // dns record
    record_type: String,
    content: String, // ipv4, ipv6
    #[serde(with = "my_custom_date_format")]
    created_at: PrimitiveDateTime,
}

async fn index(_req: Request, _ctx: RouteContext<()>) -> Result<Response> {
    console_log!("fn greet");
    Response::ok("Welcome to my worker")
}

async fn dns(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let pem_text = ctx.secret("PEM_TEXT")?.to_string();
    //let body: UpdateDnsRequestData = req.json().await?;
    let body_bytes = req.bytes().await?;
    let signature_base64 = req.headers().get("X-Signature")?.unwrap_or_default();
    if let Err(e) = signature::verify_signature(&pem_text, &body_bytes, &signature_base64).await {
        console_error!("Signature verification failed: {}", e);
        return Response::error("Invalid signature", 403);
    }
    let body: UpdateDnsRequestData = serde_json::from_slice(&body_bytes)?;
    if body.content.is_empty() {
        return Err("Content can't be empty".into());
    }
    let source_ip = req.headers().get("CF-Connecting-IP")?.unwrap_or_default();
    let token = ctx.secret("CLOUDFLARE_API_TOKEN")?.to_string();

    let zone_id = match ctx.kv(ZONES_KV)?.get(&body.zone).text().await? {
        Some(zone_id) => zone_id,
        None => {
            console_log!("kv without zone_id try get_zone_id()");
            let zone_id = cloudflare::dns::get_zone_id(&body.zone, &token).await?;
            ctx.kv(ZONES_KV)?
                .put(&body.zone, &zone_id)?
                .execute()
                .await?;
            zone_id
        }
    };

    let records_kv_key = format!("{}:{}", &body.record, &body.record_type);
    let record_id = match ctx.kv(RECORDS_KV)?.get(&records_kv_key).text().await? {
        Some(record_id) => record_id,
        None => {
            console_log!("kv without record_id try get_dns_record_id()");
            let record_id = cloudflare::dns::get_dns_record_id(
                &zone_id,
                &body.record,
                &body.record_type,
                &token,
            )
            .await?;
            ctx.kv(RECORDS_KV)?
                .put(&records_kv_key, &record_id)?
                .execute()
                .await?;
            record_id
        }
    };

    cloudflare::dns::update_dns_record(&zone_id, &record_id, &body.content, &token).await?;
    let log = format!(
        "{time} message=\"Update DNS record success.\" record={record} record_type={record_type} content={content}",
        record = &body.record,
        record_type = &body.record_type,
        content = &body.content,
        time = Date::now().as_millis().to_string()
    );
    let d1 = ctx.env.d1("ddns")?;
    cloudflare::d1::write_log(&d1, "INFO", "fn dns", &log, &source_ip).await?;

    Response::ok("DNS record updated")
}

async fn handle_errors<F>(req: Request, env: Env, future: F) -> Result<Response>
where
    F: Future<Output = Result<Response>>,
{
    let d1 = env.d1("ddns")?;
    let source_ip = req.headers().get("CF-Connecting-IP")?.unwrap_or_default();
    match future.await {
        Ok(response) => Ok(response),
        Err(err) => {
            let log = format!(
                "{} err={}",
                Date::now().as_millis().to_string(),
                err.to_string()
            );
            cloudflare::d1::write_log(&d1, "ERROR", "fn handle_errors", &log, &source_ip).await?;
            match err.to_string().as_str() {
                "Unauthorized" => Response::builder()
                    .with_header("WWW-Authenticate", r#"Basic realm="DDNS Logs""#)?
                    .error("Unauthorized", 401),
                "Zone not found" => Response::error(err.to_string(), 404),
                "Binding `CLOUDFLARE_API_TOKEN` is undefined." => {
                    Response::error(err.to_string(), 404)
                }
                "Invalid request headers" => Response::error(err.to_string(), 403),
                _ => {
                    //console_error!("Request failed: {:?}", err);
                    Response::error(err.to_string(), 500)
                }
            }
        }
    }
}

async fn app(req: Request, env: Env) -> Result<Response> {
    let result = Router::new()
        .get_async("/", index)
        .post_async("/dns", dns)
        .get_async("/logs", get_logs)
        .run(req.clone()?, env.clone());
    handle_errors(req, env, result).await
}

#[event(fetch)]
async fn fetch(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    app(req, env).await
}
