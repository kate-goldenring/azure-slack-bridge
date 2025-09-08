use serde::Serialize;
use spin_sdk::http::{IntoResponse, Method, Request, Response};
use spin_sdk::{http_component, variables};
mod azure_common_alert_schema;
use azure_common_alert_schema::AzureAlert;

// Slack message structure
#[derive(Serialize, Debug)]
struct SlackMessage {
    text: String,
}

#[http_component]
async fn handle_azure_to_slack_webhook(req: Request) -> anyhow::Result<impl IntoResponse> {
    // Get the Slack webhook URL from a dynamic application variable
    let slack_webhook_url = variables::get("slack_webhook_url")?;

    // Parse the Azure alert JSON message from the HTTP request body
    let azure_alert = match serde_json::from_slice::<AzureAlert>(req.body()) {
        Ok(alert) => alert,
        Err(e) => {
            eprintln!("Failed to parse Azure alert: {}", e);
            return Ok(Response::builder()
                .status(400)
                .body(format!("Invalid alert payload: {}", e))
                .build());
        }
    };

    // Format the origin message for Slack
    let slack_text = format_alert_message(&azure_alert);

    // Send the message to Slack
    let slack_message = SlackMessage { text: slack_text };

    let request = Request::builder()
        .method(Method::Post)
        .body(serde_json::to_vec(&slack_message)?)
        .header("Content-type", "application/json")
        .uri(slack_webhook_url)
        .build();

    // Return the result back to Azure
    let response: Response = spin_sdk::http::send(request).await?;
    if *response.status() != 200 {
        return Ok(Response::new(500, "Failed to send to Slack"));
    }
    Ok(Response::new(200, ""))
}

fn format_alert_message(alert: &AzureAlert) -> String {
    let essentials = &alert.data.essentials;
    let condition = &alert.data.alert_context.condition;

    // Extract resource name from target ID
    let resource_name = essentials
        .alert_target_ids
        .first()
        .and_then(|id| id.split('/').next_back())
        .unwrap_or("Unknown");

    let severity_emoji = alert.get_severity_emoji();

    let condition_emoji = alert.get_condition_emoji();

    let mut message = format!(
        "{} {} *{}*\n",
        severity_emoji, condition_emoji, essentials.alert_rule
    );

    if let Some(description) = &essentials.description {
        message.push_str(&format!("> *Description:* {}\n", description));
    }

    message.push_str(&format!("> *Resource:* {}\n", resource_name));
    message.push_str(&format!("> *Severity:* {}\n", essentials.severity));
    message.push_str(&format!("> *Time:* {}\n", essentials.fired_date_time));

    // Add metric details
    if let Some(metric) = condition.all_of.first() {
        message.push_str(&format!(
            "> *Metric:* {} {} {} (Current: {})\n",
            metric.metric_name, metric.operator, metric.threshold, metric.metric_value
        ));
        // Add dimensions (like StatusCode: 429)
        if let Some(dimensions) = &metric.dimensions {
            for dim in dimensions {
                message.push_str(&format!("> *{}:* {}\n", dim.name, dim.value));
            }
        }
    }
    let azure_alerts_dashboard_url =
        variables::get("azure_alerts_dashboard_url").unwrap_or_default();
    if !azure_alerts_dashboard_url.is_empty() {
        message.push_str(&format!(
            "> <{}|View in Azure>\n",
            azure_alerts_dashboard_url
        ));
    }

    message
}
