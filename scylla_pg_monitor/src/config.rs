use super::{env_var, env_var_with_defaults};

#[derive(Debug, Clone, Default)]
pub struct PGMonitorConfig {
    pub poll_interval: u64,
    pub metrics_refresh_interval: u64,
    pub metrics_host: String,
    pub metrics_path: String,
    pub metrics_port: u16,
    pub task_retention_time: i64,
}

impl PGMonitorConfig {
    pub fn from_env() -> Self {
        Self {
            poll_interval: env_var!("MONITOR_POLLING_INTERVAL_IN_SECS")
                .parse()
                .expect("u64 expected for MONITOR_POLLING_INTERVAL_IN_SECS"),
            metrics_refresh_interval: env_var_with_defaults!("MONITOR_METRICS_REFRESH_INTERVAL_IN_SECS", u64, 120),
            metrics_host: env_var_with_defaults!("MONITOR_METRICS_HOST", "0.0.0.0".to_string()),
            metrics_path: env_var_with_defaults!("MONITOR_METRICS_PATH", "/metrics".to_string()),
            metrics_port: env_var_with_defaults!("MONITOR_METRICS_PORT", u16, 9464),
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
            ("MONITOR_POLLING_INTERVAL_IN_SECS", "10"),
            ("MONITOR_METRICS_REFRESH_INTERVAL_IN_SECS", "120"),
            ("MONITOR_METRICS_HOST", "0.0.0.0"),
            ("MONITOR_METRICS_PATH", "/metrics"),
            ("MONITOR_METRICS_PORT", "9464"),
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
        assert_eq!(config.poll_interval, 10);
        assert_eq!(config.metrics_refresh_interval, 120);
        assert_eq!(config.metrics_host, "0.0.0.0".to_string());
        assert_eq!(config.metrics_path, "/metrics".to_string());
        assert_eq!(config.metrics_port, 9464);
        assert_eq!(config.task_retention_time, 8600);
        get_monitor_env_variables().iter().for_each(|(k, _)| {
            unset_env_var(k);
        });
    }

    #[test]
    #[serial]
    fn check_from_env_uses_defaults() {
        set_env_var("MONITOR_POLLING_INTERVAL_IN_SECS", "10");
        set_env_var("MONITOR_TASK_RETENTION_PERIOD_IN_SECS", "8600");

        let config = PGMonitorConfig::from_env();
        assert_eq!(config.metrics_refresh_interval, 120);
        assert_eq!(config.metrics_host, "0.0.0.0".to_string());
        assert_eq!(config.metrics_path, "/metrics".to_string());
        assert_eq!(config.metrics_port, 9464);
        assert_eq!(config.task_retention_time, 8600);

        unset_env_var("MONITOR_POLLING_INTERVAL_IN_SECS");
        unset_env_var("MONITOR_TASK_RETENTION_PERIOD_IN_SECS");
    }
}
