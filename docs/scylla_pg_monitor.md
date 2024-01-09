# Scylla PG Monitor

It's monitor binary, that is bundled as a docker image. This is a standalone necessary component that needs to run to make sure tasks are not stuck indefinitely and also for cleaning up older tasks based on retention time.

Docker Image would need below environment variables:

```text
PG_HOST=
PG_PORT=
PG_USER=
PG_PASSWORD=
PG_DATABASE=
PG_POOL_SIZE=

MONITOR_POLLING_INTERVAL_IN_SECS=
MONITOR_TASK_RETENTION_PERIOD_IN_SECS=

RUST_LOG=
```

