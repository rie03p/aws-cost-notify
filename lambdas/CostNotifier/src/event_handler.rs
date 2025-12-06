use aws_config::BehaviorVersion;
use aws_sdk_costexplorer::{types::{DateInterval, Granularity, GroupDefinition, GroupDefinitionType}, Client as CostExplorerClient};
use chrono::{Duration, Utc};
use lambda_runtime::{tracing, Error, LambdaEvent};
use aws_lambda_events::event::eventbridge::EventBridgeEvent;
use serde::Serialize;
use std::env;

#[derive(Serialize)]
struct DiscordEmbed {
    title: String,
    description: String,
    color: u32,
    fields: Vec<DiscordField>,
}

#[derive(Serialize)]
struct DiscordField {
    name: String,
    value: String,
    inline: bool,
}

#[derive(Serialize)]
struct DiscordWebhook {
    embeds: Vec<DiscordEmbed>,
}

/// AWS OrganizationのアカウントごとのコストをDiscordに通知
pub(crate) async fn function_handler(event: LambdaEvent<EventBridgeEvent>) -> Result<(), Error> {
    let payload = event.payload;
    tracing::info!("Payload: {:?}", payload);

    // Discord Webhook URLを環境変数から取得
    let webhook_url = env::var("DISCORD_WEBHOOK_URL")
        .map_err(|_| "DISCORD_WEBHOOK_URL environment variable not set")?;

    // AWS Cost Explorer クライアントを初期化
    let config = aws_config::load_defaults(BehaviorVersion::latest()).await;
    let client = CostExplorerClient::new(&config);

    // 日付範囲を設定 (過去7日間)
    let end_date = Utc::now();
    let start_date = end_date - Duration::days(7);
    
    let time_period = DateInterval::builder()
        .start(start_date.format("%Y-%m-%d").to_string())
        .end(end_date.format("%Y-%m-%d").to_string())
        .build()
        .map_err(|e| format!("Failed to build DateInterval: {}", e))?;

    // アカウントごとにグループ化してコストを取得
    let group_by = GroupDefinition::builder()
        .r#type(GroupDefinitionType::Dimension)
        .key("LINKED_ACCOUNT")
        .build();

    let response = client
        .get_cost_and_usage()
        .time_period(time_period)
        .granularity(Granularity::Daily)
        .metrics("UnblendedCost")
        .group_by(group_by)
        .send()
        .await
        .map_err(|e| format!("Failed to get cost and usage: {}", e))?;

    // コストデータを整形
    let mut fields = Vec::new();
    let mut total_cost = 0.0;

    // results_by_time は &[ResultByTime] を返す
    for result in response.results_by_time() {
        // groups も &[Group] を返す
        for group in result.groups() {
            let account_id = group
                .keys()
                .first()
                .map(|s| s.as_str())
                .unwrap_or("Unknown");

            let cost = group
                .metrics()
                .and_then(|m| m.get("UnblendedCost"))
                .and_then(|metric| metric.amount())
                .and_then(|amount| amount.parse::<f64>().ok())
                .unwrap_or(0.0);

            if cost > 0.01 {
                fields.push(DiscordField {
                    name: format!("Account: {}", account_id),
                    value: format!("${:.2}", cost),
                    inline: true,
                });
                total_cost += cost;
            }
        }
    }
    
    // 合計を追加
    fields.push(DiscordField {
        name: "**Total**".to_string(),
        value: format!("**${:.2}**", total_cost),
        inline: false,
    });

    // Discord Webhookに送信
    let embed = DiscordEmbed {
        title: "AWS Organization - Daily Cost Report".to_string(),
        description: format!(
            "Cost report for {} to {}",
            start_date.format("%Y-%m-%d"),
            end_date.format("%Y-%m-%d")
        ),
        color: 0x00ff00,
        fields,
    };

    let webhook_payload = DiscordWebhook {
        embeds: vec![embed],
    };

    let client = reqwest::Client::new();
    let res = client
        .post(&webhook_url)
        .json(&webhook_payload)
        .send()
        .await
        .map_err(|e| format!("Failed to send Discord webhook: {}", e))?;

    if !res.status().is_success() {
        let status = res.status();
        let body = res.text().await.unwrap_or_default();
        return Err(format!("Discord webhook failed with status {}: {}", status, body).into());
    }

    tracing::info!("Successfully sent cost notification to Discord");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use lambda_runtime::{Context, LambdaEvent};

    #[tokio::test]
    async fn test_event_handler() {
        // Note: This test requires AWS credentials and Discord webhook URL
        // Skip in CI/CD or set appropriate environment variables
        if env::var("DISCORD_WEBHOOK_URL").is_err() {
            println!("Skipping test: DISCORD_WEBHOOK_URL not set");
            return;
        }

        let event = LambdaEvent::new(EventBridgeEvent::default(), Context::default());
        let response = function_handler(event).await;
        assert!(response.is_ok());
    }
}
