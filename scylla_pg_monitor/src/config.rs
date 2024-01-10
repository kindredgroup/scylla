use super::env_var;

#[derive(Debug, Clone, Default)]
pub struct PGMonitorConfig {
    pub poll_interval: u64,
    pub task_retention_time: i64,
}

impl PGMonitorConfig {
    pub fn from_env() -> Self {
        Self {
            poll_interval: env_var!("MONITOR_POLLING_INTERVAL_IN_SECS")
                .parse()
                .expect("u64 expected for MONITOR_POLLING_INTERVAL_IN_SECS"),
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
        let env_hashmap = [("MONITOR_POLLING_INTERVAL_IN_SECS", "10"), ("MONITOR_TASK_RETENTION_PERIOD_IN_SECS", "8600")];
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
        assert_eq!(config.task_retention_time, 8600);
        get_monitor_env_variables().iter().for_each(|(k, _)| {
            unset_env_var(k);
        });
    }
}
