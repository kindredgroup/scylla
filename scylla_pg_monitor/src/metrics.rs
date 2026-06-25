use std::{
    collections::BTreeSet,
    error::Error,
    fmt::{self, Display, Formatter},
    sync::Arc,
};

use opentelemetry::{global, metrics::Gauge, KeyValue};
use opentelemetry_otlp::WithExportConfig;
use tokio::sync::RwLock;

const METRIC_METER_NAME: &str = "scylla_pg_monitor";
const TASK_COUNT_METRIC_NAME: &str = "scylla_task_count";
const TASK_STATUS_LABEL: &str = "status";

#[derive(Clone)]
pub struct MetricsState {
    task_counts: Gauge<i64>,
    known_statuses: Arc<RwLock<BTreeSet<String>>>,
}

impl Default for MetricsState {
    fn default() -> Self {
        let task_counts = global::meter(METRIC_METER_NAME).i64_gauge(TASK_COUNT_METRIC_NAME).with_unit("tasks").build();

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

pub fn init_otel_metrics(grpc_endpoint: Option<String>) -> Result<(), OtelInitError> {
    if let Some(grpc_endpoint) = grpc_endpoint {
        let otel_exporter = opentelemetry_otlp::MetricExporter::builder()
            .with_tonic()
            .with_endpoint(grpc_endpoint)
            .with_protocol(opentelemetry_otlp::Protocol::Grpc)
            .build()
            .map_err(|metric_error| OtelInitError {
                kind: InitErrorType::MetricError,
                reason: "Unable to initialise metrics exporter".into(),
                cause: Some(format!("{metric_error:?}")),
            })?;

        let provider = opentelemetry_sdk::metrics::SdkMeterProvider::builder()
            .with_periodic_exporter(otel_exporter)
            .build();

        log::info!("OTEL metrics provider initialised with endpoint");
        global::set_meter_provider(provider);
    } else {
        log::info!("No OTEL endpoint provided, metrics will default to NoopMeterProvider");
    }

    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OtelInitError {
    pub kind: InitErrorType,
    pub reason: String,
    pub cause: Option<String>,
}

impl Display for OtelInitError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Error initialising OTEL telemetry: '{}'. Reason: {} Cause: {:?}",
            self.kind, self.reason, self.cause
        )
    }
}

impl Error for OtelInitError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InitErrorType {
    MetricError,
}

impl Display for InitErrorType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::MetricError => write!(f, "MetricError"),
        }
    }
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
        init_otel_metrics(None).expect("otel metrics should allow a missing endpoint");
    }
}
