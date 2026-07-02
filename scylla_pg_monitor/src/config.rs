use super::{env_var, env_var_with_defaults};

#[derive(Debug, Clone, Default)]
pub struct PGMonitorConfig {
    pub otel_grpc_endpoint: Option<String>,
    pub poll_interval: u64,
    pub metrics_refresh_interval: u64,
    pub task_retention_time: i64,
}

impl PGMonitorConfig {
    pub fn from_env() -> Self {
        Self {
            otel_grpc_endpoint: env_var_with_defaults!("OTEL_EXPORTER_OTLP_ENDPOINT", Option::<String>),
            poll_interval: env_var_with_defaults!("MONITOR_POLLING_INTERVAL_IN_SECS", u64, 5),
            metrics_refresh_interval: env_var_with_defaults!("MONITOR_METRICS_REFRESH_INTERVAL_IN_SECS", u64, 120),
            task_retention_time: env_var!("MONITOR_TASK_RETENTION_PERIOD_IN_SECS")
                .parse()
                .expect("i64 expected for MONITOR_TASK_RETENTION_PERIOD_IN_SECS"),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, env};

    use serial_test::serial;

    use super::*;

    fn set_env_var(key: &str, value: &str) {
        env::set_var(key, value)
    }

    fn unset_env_var(key: &str) {
        env::remove_var(key)
    }

    fn get_monitor_env_variables() -> HashMap<&'static str, &'static str> {
        let env_hashmap = [
            ("OTEL_EXPORTER_OTLP_ENDPOINT", "http://localhost:4317"),
            ("MONITOR_POLLING_INTERVAL_IN_SECS", "10"),
            ("MONITOR_METRICS_REFRESH_INTERVAL_IN_SECS", "120"),
            ("MONITOR_TASK_RETENTION_PERIOD_IN_SECS", "8600"),
        ];
        HashMap::from(env_hashmap)
    }

    #[test]
    #[serial]
    fn check_from_env() {
        get_monitor_env_variables().iter().for_each(|(k, v)| {
            set_env_var(k, v);
        });
        let config = PGMonitorConfig::from_env();
        assert_eq!(config.otel_grpc_endpoint, Some("http://localhost:4317".to_string()));
        assert_eq!(config.poll_interval, 10);
        assert_eq!(config.metrics_refresh_interval, 120);
        assert_eq!(config.task_retention_time, 8600);
        get_monitor_env_variables().iter().for_each(|(k, _)| {
            unset_env_var(k);
        });
    }

    #[test]
    #[serial]
    fn check_from_env_uses_defaults() {
        set_env_var("MONITOR_TASK_RETENTION_PERIOD_IN_SECS", "8600");

        let config = PGMonitorConfig::from_env();
        assert_eq!(config.otel_grpc_endpoint, None);
        assert_eq!(config.poll_interval, 5);
        assert_eq!(config.metrics_refresh_interval, 120);
        assert_eq!(config.task_retention_time, 8600);

        unset_env_var("MONITOR_TASK_RETENTION_PERIOD_IN_SECS");
    }
}
