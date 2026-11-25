use crate::store::Store;
use chrono::Utc;
use sqlx::Row;

pub fn since_duration(text: &str) -> anyhow::Result<chrono::Duration> {
    let (number, multiplier) = if let Some(value) = text.strip_suffix('m') {
        (value, 60)
    } else if let Some(value) = text.strip_suffix('h') {
        (value, 3600)
    } else if let Some(value) = text.strip_suffix('d') {
        (value, 86400)
    } else {
        anyhow::bail!("since must be a positive duration such as 30m, 24h or 7d");
    };
    let number: i64 = number
        .parse()
        .map_err(|_| anyhow::anyhow!("invalid report duration"))?;
    let seconds = number
        .checked_mul(multiplier)
        .filter(|value| *value > 0 && *value <= 10 * 366 * 86400)
        .ok_or_else(|| anyhow::anyhow!("report duration must be positive and at most ten years"))?;
    Ok(chrono::Duration::seconds(seconds))
}
pub fn percentile95(values: &[i64]) -> Option<i64> {
    if values.is_empty() {
        return None;
    }
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    Some(sorted[(sorted.len() * 95).div_ceil(100) - 1])
}
pub async fn print(
    store: &Store,
    since: &str,
    name: Option<&str>,
    security: bool,
) -> anyhow::Result<()> {
    if security {
        let findings = sqlx::query("SELECT t.name,f.rule,f.severity,f.detail FROM security_findings f JOIN targets t ON t.id=f.target_id WHERE t.active=1 AND f.resolved_at IS NULL AND (? IS NULL OR t.name=?) ORDER BY CASE f.severity WHEN 'high' THEN 0 WHEN 'medium' THEN 1 WHEN 'low' THEN 2 ELSE 3 END,t.name")
            .bind(name).bind(name).fetch_all(&store.pool).await?;
        println!("TARGET               SEVERITY  RULE                 DETAIL");
        for row in &findings {
            println!(
                "{:<20} {:<9} {:<20} {}",
                row.get::<String, _>("name"),
                row.get::<String, _>("severity"),
                row.get::<String, _>("rule"),
                row.get::<String, _>("detail")
            );
        }
        if findings.is_empty() {
            println!("No recorded unresolved security findings.");
        }
        return Ok(());
    }
    let cutoff = (Utc::now() - since_duration(since)?).to_rfc3339();
    println!("TARGET               SAMPLES  SAMPLE UP %   AVG MS   P95 MS  INCIDENTS");
    for target in store.statuses().await? {
        if name.is_some_and(|name| name != target.name) {
            continue;
        }
        let rows = sqlx::query("SELECT c.ok,c.latency_ms FROM checks c JOIN targets t ON c.target_id=t.id WHERE t.name=? AND c.checked_at>=?").bind(&target.name).bind(&cutoff).fetch_all(&store.pool).await?;
        let success: Vec<i64> = rows
            .iter()
            .filter(|row| row.get::<bool, _>("ok"))
            .map(|row| row.get("latency_ms"))
            .collect();
        let availability = if rows.is_empty() {
            "n/a".into()
        } else {
            format!("{:.2}", success.len() as f64 / rows.len() as f64 * 100.0)
        };
        let average = if success.is_empty() {
            "n/a".into()
        } else {
            format!(
                "{:.0}",
                success.iter().sum::<i64>() as f64 / success.len() as f64
            )
        };
        let p95 = percentile95(&success)
            .map(|value| value.to_string())
            .unwrap_or_else(|| "n/a".into());
        let incidents: i64 = sqlx::query_scalar("SELECT count(*) FROM incidents i JOIN targets t ON t.id=i.target_id WHERE t.name=? AND (i.resolved_at IS NULL OR i.resolved_at>=?)").bind(&target.name).bind(&cutoff).fetch_one(&store.pool).await?;
        println!(
            "{:<20} {:>7} {:>12} {:>8} {:>8} {:>10}",
            target.name,
            rows.len(),
            availability,
            average,
            p95,
            incidents
        );
    }
    println!("Availability is sample-based; unobserved periods are not assumed healthy. Latencies include successful checks only.");
    Ok(())
}
