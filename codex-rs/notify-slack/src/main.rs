use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use serde::Deserialize;
use serde::Serialize;
use std::env;

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
enum UserNotification {
    #[serde(rename_all = "kebab-case")]
    AgentTurnComplete {
        turn_id: String,
        input_messages: Vec<String>,
        last_assistant_message: Option<String>,
    },
}

#[derive(Debug, Serialize)]
struct SlackPayload<'a> {
    text: &'a str,
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.is_empty() {
        return Err(anyhow!(
            "usage: codex-notify-slack '{json-payload-from-codex}' [--webhook <url>]"
                .replace("{", "{{")
                .replace("}", "}}")
        ));
    }

    let mut webhook = env::var("SLACK_WEBHOOK_URL").ok();
    if let Some(pos) = args.iter().position(|a| a == "--webhook")
        && let Some(val) = args.get(pos + 1)
    {
        webhook = Some(val.clone());
        args.drain(pos..=pos + 1);
    }

    let json_arg = args.remove(0);
    let note: UserNotification = serde_json::from_str(&json_arg)
        .with_context(|| "invalid JSON payload passed from cxplus notifier")?;

    let text = match note {
        UserNotification::AgentTurnComplete {
            turn_id,
            input_messages,
            last_assistant_message,
        } => {
            let first = input_messages.first().map(String::as_str).unwrap_or("");
            let last = last_assistant_message.unwrap_or_default();
            format!("cxplus: turn {turn_id} complete\n• prompt: {first}\n• last: {last}")
        }
    };

    let webhook =
        webhook.ok_or_else(|| anyhow!("SLACK_WEBHOOK_URL not set and no --webhook provided"))?;

    let client = reqwest::Client::new();
    let resp = client
        .post(&webhook)
        .json(&SlackPayload { text: &text })
        .send()
        .await
        .context("failed to send Slack webhook request")?;

    if !resp.status().is_success() {
        return Err(anyhow!("slack webhook returned status {}", resp.status()));
    }
    Ok(())
}
