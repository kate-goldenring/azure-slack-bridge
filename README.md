# Azure Slack Bridge

A Spin application middleware that transforms Azure Monitor alerts into beautifully formatted Slack notifications.

## Why Azure Slack Bridge?

Azure Monitor's native webhook support does not integrate with Slack because Azure sends JSON in its own format while Slack expects a different structure. This Spin application is middleware that sits between Azure and Slack, parsing Azure's Common Alert Schema and transforming it into readable Slack messages with emojis and proper formatting. It can be run locally (say with Tailscale in front of it) or deployed to any platform that hosts Spin applications, such as [Fermyon Wasm Functions](https://www.fermyon.com/wasm-functions), [Fermyon Cloud](https://www.fermyon.com/cloud), or a [SpinKube](https://www.spinkube.dev/) cluster.

## ✨ Features

- **Format Transformation**: Converts Azure's [common alert schema JSON](https://learn.microsoft.com/en-us/azure/azure-monitor/alerts/alerts-common-schema#sample-alert-payload) into Slack-friendly messages
- **High Performance**: Built with Spin as an instantly executable serverless WebAssembly function

## 🎯 Sample Output

As an example, a Cosmos DB administrator may want to [alert when a certain threshold of 429 errors occurs](https://learn.microsoft.com/en-us/azure/cosmos-db/create-alerts). The following is an example message output:

```md
⚠️ 🔴 *Exceeded Threshold of Too Many Request Errors (429s)*
> *Description:* More than 100 Too Many Request (429s) Errors in the past 5 minutes. Cosmos DB usage is exceeding the configured allowed rate. Consider increasing RUs on the database.
> *Resource:* my-resource-group
> *Severity:* Sev2
> *Time:* 2025-09-08T21:16:41.9914734Z
> *Metric:* TotalRequests GreaterThan 100 (Current: 968)
> *StatusCode:* 429
> <https://portal.azure.com/myorg/resource/subscriptions/my-subscription/resourceGroups/my-resource-group/providers/Microsoft.DocumentDB/databaseAccounts/my-database-name/alerts|View in Azure>
```

## 🚀 Quick Start

After [installing Spin](https://spinframework.dev/v3/install) and [creating a Slack incoming Webhook](#slack-webhook-setup), you can test your application locally. First, build and run the application, setting the URL to your Slack Webhook in a Spin Variable using the environment variable provider.

```bash
git clone https://github.com/kate-goldenring/azure-slack-bridge
cd azure-slack-bridge
SPIN_VARIABLE_SLACK_WEBHOOK_URL="https://hooks.slack.com/services/<...> spin build --up
```

Test your connection to Slack with an example Azure alert payload:

```bash
curl -X POST \
  -H "Content-Type: application/json" \
  -d @sample-alert.json \
  localhost:3000
```

## 🏗️ Architecture

```
Azure Monitor → Azure Slack Bridge → Slack
     │                 │              │
   (Complex          (Parses &      (Simple
    JSON)           Transforms)     Message)
```

## 🔧 Configuration

### Spin Variables

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `SLACK_WEBHOOK_URL` | ✅ Yes | - | Your Slack webhook URL |
| `AZURE_ALERTS_DASHBOARD_URL` | ✅ Yes| - | URL of alerts page for incident response |

## 📦 Deploying Your Spin Application

Spin application can be deployed to any Spin platform, oftentimes using Spin CLI plugins. The following are three options:

[Fermyon Wasm Functions](https://www.fermyon.com/wasm-functions):

```sh
spin aka deploy
```

[Fermyon Cloud](https://www.fermyon.com/cloud):

```sh
spin cloud deploy
```

Your [SpinKube](https://www.spinkube.dev/) cluster:

```sh
spin registry push ttl.sh/azure-slack-bridge:24h
spin kube scaffold -f ttl.sh/azure-slack-bridge:24h | kubectl apply -f -
```

## ⚙️ Setup

### Slack Webhook Setup

1. Go to your Slack workspace settings
2. Create a new app or use an existing one
3. Enable "Incoming Webhooks"
4. Create a webhook for your desired channel
5. Copy the webhook URL for the `SLACK_WEBHOOK_URL` environment variable

## Setting Up the Alert Pipeline

1. Create a [Slack Incoming Webhook](#slack-webhook-setup)
2. Deploy your Spin application, setting the `slack_webhook_url` variable to be the URL of the previously created Slack webhook
3. Create an Azure Action Group with a Webhook Action type. Set the URI of the Webhook to your Spin application and *enable the common alert schema*.
4. Create an Azure Alert Rule and connect it to the previously created Action Group
