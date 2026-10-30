use serde::{Deserialize, Serialize};
use worker::*;

#[derive(Deserialize, Serialize)]
pub struct DDNSLog {
    id: u32,
    level: String,
    target: String,
    message: String,
    source_ip: String,
    created_at: String,
}

pub async fn write_log(
    db: &D1Database,
    level: &str,
    target: &str,
    message: &str,
    source_ip: &str,
) -> Result<()> {
    let statement = db.prepare(
        "INSERT INTO ddns_logs (level, target, message, source_ip)
         VALUES (?1, ?2, ?3, ?4)",
    );

    let query = statement.bind(&[
        level.into(),
        target.into(),
        message.into(),
        source_ip.into(),
    ])?;

    query.run().await?;

    Ok(())
}

pub async fn get_logs(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    crate::auth::basic_auth(&req, &ctx).await.map_err(|e| {
        console_debug!("{}", e);
        Error::RustError("Unauthorized".into())
    })?;
    let d1 = ctx.env.d1("ddns")?;
    let statement = d1.prepare("SELECT * FROM ddns_logs ORDER BY created_at DESC limit 100");
    let query = statement.bind(&[])?;
    let result = query.all().await?;
    let results = result.results::<DDNSLog>()?;
    let mut output = String::new();

    for log in &results {
        output.push_str(&format!(
            "{} | [{:^5}] | [{:^20}] | [{:^39}] | {}\n",
            format_local_time(&log.created_at),
            log.level,
            log.target,
            log.source_ip,
            log.message,
        ));
    }
    Response::ok(output)
}

fn format_local_time(datetime: &str) -> String {
    let format = time::format_description::parse_borrowed::<3>(
        "[year]-[month]-[day] [hour]:[minute]:[second]",
    )
    .unwrap();

    let offset = time::UtcOffset::from_hms(8, 0, 0).unwrap();

    time::PrimitiveDateTime::parse(datetime, &format)
        .unwrap()
        .assume_utc()
        .to_offset(offset)
        .format(&format)
        .unwrap()
}
