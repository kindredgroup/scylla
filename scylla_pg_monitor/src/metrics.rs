use std::{
    collections::BTreeSet,
    sync::Arc,
};

use opentelemetry::{global, metrics::Gauge, KeyValue};
use opentelemetry_otlp::WithExportConfig;
use tokio::sync::RwLock;

const METRIC_METER_NAME: &str = "scylla_pg_monitor";
const TASK_COUNT_METRIC_NAME: &str = "scylla_tasks";
const TASK_STATUS_LABEL: &str = "status";

#[derive(Clone)]
pub struct MetricsState {
    task_counts: Gauge<i64>,
    known_statuses: Arc<RwLock<BTreeSet<String>>>,
}

impl Default for MetricsState {
    fn default() -> Self {
        let task_counts = global::meter(METRIC_METER_NAME).i64_gauge(TASK_COUNT_METRIC_NAME).with_unit("1").build();

        Self {
            task_counts,
            known_statuses: Arc::new(RwLock::new(BTreeSet::new())),
        }
    }
}

impl MetricsState {
    pub async fn update_task_counts(&self, counts: Vec<(String, i64)>) {
        let next_statuses: BTreeSet<_> = counts.iter().map(|(status, _)| status.clone()).collect();
        let mut known_statuses = self.known_statuses.write().await;

        for status in known_statuses.difference(&next_statuses) {
            self.record_task_count(status, 0);
        }

        for (status, count) in counts {
            self.record_task_count(&status, count);
        }

        *known_statuses = next_statuses;
    }

    fn record_task_count(&self, status: &str, count: i64) {
        self.task_counts.record(count, &[KeyValue::new(TASK_STATUS_LABEL, status.to_string())]);
    }
}

fn is_valid_otel_grpc_endpoint(endpoint: &str) -> bool {
    let rest = endpoint
        .strip_prefix("http://")
        .or_else(|| endpoint.strip_prefix("https://"));

    rest.is_some_and(|host| !host.is_empty())
}

pub fn init_otel_metrics(grpc_endpoint: Option<String>) {
    let Some(raw_endpoint) = grpc_endpoint else {
        log::info!("No OTEL endpoint provided, metrics will default to NoopMeterProvider");
        return;
    };

    if raw_endpoint.trim().is_empty() {
        log::error!("OTEL_EXPORTER_OTLP_ENDPOINT is set but empty; metrics will default to NoopMeterProvider");
        return;
    }

    let endpoint = raw_endpoint.trim().to_string();
    if !is_valid_otel_grpc_endpoint(&endpoint) {
        log::error!(
            "OTEL_EXPORTER_OTLP_ENDPOINT is invalid ({endpoint:?}); expected an http:// or https:// URL; metrics will default to NoopMeterProvider"
        );
        return;
    }

    let otel_exporter = match opentelemetry_otlp::MetricExporter::builder()
        .with_tonic()
        .with_endpoint(endpoint.clone())
        .with_protocol(opentelemetry_otlp::Protocol::Grpc)
        .build()
    {
        Ok(exporter) => exporter,
        Err(metric_error) => {
            log::error!(
                "Unable to initialise OTEL metrics exporter for endpoint {endpoint:?}: {metric_error:?}; metrics will default to NoopMeterProvider"
            );
            return;
        }
    };

    let provider = opentelemetry_sdk::metrics::SdkMeterProvider::builder()
        .with_periodic_exporter(otel_exporter)
        .build();

    log::info!("OTEL metrics provider initialised with endpoint {endpoint:?}");
    global::set_meter_provider(provider);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn tracks_known_statuses_after_updates() {
        let state = MetricsState::default();
        state.update_task_counts(vec![("ready".to_string(), 3), ("running".to_string(), 1)]).await;

        let known_statuses = state.known_statuses.read().await;
        assert!(known_statuses.contains("ready"));
        assert!(known_statuses.contains("running"));
    }

    #[tokio::test]
    async fn removes_stale_statuses_from_tracking() {
        let state = MetricsState::default();
        state.update_task_counts(vec![("ready".to_string(), 3), ("running".to_string(), 1)]).await;
        state.update_task_counts(vec![("ready".to_string(), 5)]).await;

        let known_statuses = state.known_statuses.read().await;
        assert!(known_statuses.contains("ready"));
        assert!(!known_statuses.contains("running"));
    }

    #[test]
    fn initialises_noop_metrics_without_endpoint() {
        init_otel_metrics(None);
    }

    #[test]
    fn initialises_noop_metrics_with_empty_endpoint() {
        init_otel_metrics(Some(String::new()));
        init_otel_metrics(Some("   ".to_string()));
    }

    #[test]
    fn initialises_noop_metrics_with_invalid_endpoint() {
        init_otel_metrics(Some("not-a-url".to_string()));
        init_otel_metrics(Some("http://".to_string()));
    }

    #[test]
    fn validates_otel_grpc_endpoint() {
        assert!(is_valid_otel_grpc_endpoint("http://localhost:4317"));
        assert!(is_valid_otel_grpc_endpoint("https://collector.example.com:4317"));
        assert!(!is_valid_otel_grpc_endpoint(""));
        assert!(!is_valid_otel_grpc_endpoint("http://"));
        assert!(!is_valid_otel_grpc_endpoint("ftp://localhost:4317"));
        assert!(!is_valid_otel_grpc_endpoint("localhost:4317"));
    }
}
