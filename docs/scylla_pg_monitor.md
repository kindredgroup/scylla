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

OTEL_EXPORTER_OTLP_ENDPOINT=
MONITOR_POLLING_INTERVAL_IN_SECS=
MONITOR_METRICS_REFRESH_INTERVAL_IN_SECS=120
MONITOR_TASK_RETENTION_PERIOD_IN_SECS=

RUST_LOG=
```

Metrics exported by the monitor include:

- `scylla_task_count{status="..."}` for the current number of tasks per status

The monitor flow:

- metrics are recorded through the OpenTelemetry metrics API
- when `OTEL_EXPORTER_OTLP_ENDPOINT` is set, they are exported periodically over OTLP gRPC
- when the endpoint is not set, metrics default to the noop provider
