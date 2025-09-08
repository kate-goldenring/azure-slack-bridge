// Types for Azure Common Alert Schema
// See: https://learn.microsoft.com/en-us/azure/azure-monitor/alerts/alerts-common-schema
use serde::Deserialize;

/// Azure Monitor Common Alert Schema root structure
#[derive(Deserialize, Debug)]
pub struct AzureAlert {
    #[serde(rename = "schemaId")]
    pub schema_id: String,
    pub data: AlertData,
}

/// Main alert data container
#[derive(Deserialize, Debug)]
pub struct AlertData {
    pub essentials: Essentials,
    #[serde(rename = "alertContext")]
    pub alert_context: AlertContext,
}

/// Essential alert information
#[derive(Deserialize, Debug)]
pub struct Essentials {
    #[serde(rename = "alertId")]
    pub alert_id: String,
    #[serde(rename = "alertRule")]
    pub alert_rule: String,
    pub severity: String,
    #[serde(rename = "monitorCondition")]
    pub monitor_condition: String,
    #[serde(rename = "firedDateTime")]
    pub fired_date_time: String,
    pub description: Option<String>,
    #[serde(rename = "alertTargetIDs")]
    pub alert_target_ids: Vec<String>,
    #[serde(rename = "resolvedDateTime")]
    pub resolved_date_time: Option<String>,
    #[serde(rename = "signalType")]
    pub signal_type: Option<String>,
    #[serde(rename = "monitoringService")]
    pub monitoring_service: Option<String>,
    #[serde(rename = "investigationLink")]
    pub investigation_link: Option<String>,
}

/// Alert context containing condition details
#[derive(Deserialize, Debug)]
pub struct AlertContext {
    pub condition: Condition,
    #[serde(rename = "conditionType")]
    pub condition_type: Option<String>,
}

/// Condition details for the alert
#[derive(Deserialize, Debug)]
pub struct Condition {
    #[serde(rename = "allOf")]
    pub all_of: Vec<MetricCondition>,
    #[serde(rename = "windowSize")]
    pub window_size: Option<String>,
}

/// Individual metric condition
#[derive(Deserialize, Debug)]
pub struct MetricCondition {
    #[serde(rename = "metricName")]
    pub metric_name: String,
    #[serde(rename = "metricNamespace")]
    pub metric_namespace: Option<String>,
    pub operator: String,
    pub threshold: String,
    #[serde(rename = "metricValue")]
    pub metric_value: f64,
    #[serde(rename = "timeAggregation")]
    pub time_aggregation: Option<String>,
    pub dimensions: Option<Vec<Dimension>>,
}

/// Metric dimension (e.g., StatusCode: 429)
#[derive(Deserialize, Debug)]
pub struct Dimension {
    pub name: String,
    pub value: String,
}

impl AzureAlert {
    /// Extract resource name from the first target ID
    pub fn get_resource_name(&self) -> &str {
        self.data
            .essentials
            .alert_target_ids
            .first()
            .and_then(|id| id.split('/').next_back())
            .unwrap_or("Unknown")
    }

    /// Get severity emoji based on severity level
    pub fn get_severity_emoji(&self) -> &str {
        match self.data.essentials.severity.as_str() {
            "Sev0" => "🔥",
            "Sev1" => "🚨",
            "Sev2" => "⚠️",
            "Sev3" => "ℹ️",
            "Sev4" => "📢",
            _ => "📢",
        }
    }

    /// Get condition emoji based on monitor condition
    pub fn get_condition_emoji(&self) -> &str {
        match self.data.essentials.monitor_condition.as_str() {
            "Fired" => "🔴",
            "Resolved" => "✅",
            _ => "🟡",
        }
    }

    /// Check if this is a resolved alert
    pub fn is_resolved(&self) -> bool {
        self.data.essentials.monitor_condition == "Resolved"
    }

    /// Get the first metric condition (most common case)
    pub fn get_primary_metric(&self) -> Option<&MetricCondition> {
        self.data.alert_context.condition.all_of.first()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_azure_alert_parsing() {
        // From docs sample: https://learn.microsoft.com/en-us/azure/azure-monitor/alerts/alerts-common-schema#sample-alert-payload
        let json = r#"
        {
  "schemaId": "azureMonitorCommonAlertSchema",
  "data": {
    "essentials": {
      "alertId": "/subscriptions/<subscription ID>/providers/Microsoft.AlertsManagement/alerts/aaaa0a0a-bb1b-cc2c-dd3d-eeeeee4e4e4e",
      "alertRule": "WCUS-R2-Gen2",
      "severity": "Sev2",
      "signalType": "Metric",
      "monitorCondition": "Fired",
      "monitoringService": "Platform",
      "alertTargetIDs": [
        "/subscriptions/<subscription ID>/resourcegroups/pipelinealertrg/providers/microsoft.compute/virtualmachines/wcus-r2-gen2"
      ],
      "configurationItems": [
        "wcus-r2-gen2"
      ],
      "originAlertId": "3f2d4487-b0fc-4125-8bd5-7ad17384221e_PipeLineAlertRG_microsoft.insights_metricAlerts_WCUS-R2-Gen2_-117781227",
      "firedDateTime": "2019-03-22T13:58:24.3713213Z",
      "resolvedDateTime": "2019-03-22T14:03:16.2246313Z",
      "description": "Too many requests",
      "essentialsVersion": "1.0",
      "alertContextVersion": "1.0"
    },
    "alertContext": {
      "properties": null,
      "conditionType": "SingleResourceMultipleMetricCriteria",
      "condition": {
        "windowSize": "PT5M",
        "allOf": [
          {
            "metricName": "Percentage CPU",
            "metricNamespace": "Microsoft.Compute/virtualMachines",
            "operator": "GreaterThan",
            "threshold": "25",
            "timeAggregation": "Average",
            "dimensions": [
              {
                "name": "ResourceId",
                "value": "3efad9dc-3d50-4eac-9c87-8b3fd6f97e4e"
              }
            ],
            "metricValue": 7.727
          }
        ]
      }
    },
    "customProperties": {
      "Key1": "Value1",
      "Key2": "Value2"
    }
  }
}"#;

        let alert: AzureAlert = serde_json::from_str(json).unwrap();

        assert_eq!(alert.data.essentials.alert_rule, "WCUS-R2-Gen2");
        assert_eq!(alert.get_resource_name(), "wcus-r2-gen2");
        assert_eq!(alert.get_severity_emoji(), "⚠️");
        assert_eq!(alert.get_condition_emoji(), "🔴");
        assert!(!alert.is_resolved());
    }
}
