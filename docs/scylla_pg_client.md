# scylla_pg_client

This is postgres Node.js binding for scylla task scheduler

## How to Use

### Initiate

Initiate instance of Scylla of passing DBConfig. We can use this instance to interact with Scylla tasks at later stage.

```typescript
type DbConfig = {
pgHost: string
pgPort: number
pgUser: string
pgPassword: string
pgDatabase: string
};

let sc = await Scylla.initiate({
      pgHost: "127.0.0.1",
      pgPort: 5432,
      pgUser: "admin",
      pgPassword: "admin",
      pgDatabase: "scylla",
      pgPoolSize: 50
    });

```

### Add Tasks

Queue is Logical division of tasks. Workers can choose tasks from certain queue. Highest priority tasks will be leased first.

```typescript
  let atm = {
    rn: "4b8d323c-19ab-470f-b7c8-d0380b91ca3a",
    queue: "task_queue",
    priority: 0.4,
    spec: {a: 1, b: 2}
  }
  let task_added = await sc.addTask(atm);

```
### Lease N Tasks

This will lease 3 tasks based on time and priority in descending order. WorkerId will be assigned to it and last argument is taskTimeOutInSecs. Worker needs to send heartbeat before that otherwise it will be picked by monitor and reset to ready state.
Task timeout is optional. Default value is 10 seconds.
```typescript
let task_added = await sc.leaseNTasks("task_queue", 3, "worker_id", 10);
```
### Sending Heart beat

This process is essential to let others know that task is still being processed and optionally progress can be updated by worker.
Again taskTimeOutInSecs is optional, if skipped it will be set to default to 10 seconds.

```typescript
let task = await sc.heartBeatTask("4b8d323c-19ab-470f-b7c8-d0380b91ca3a", 0.2, 20);
```

### Complete Task

Once task is completed, worker can complete the task. So it can be removed from the queue based on `MONITOR_TASK_RETENTION_PERIOD_IN_SECS` in monitor.

```typescript
let task = await sc.completeTask("4b8d323c-19ab-470f-b7c8-d0380b91ca3a");
```

### Cancel Task

In case task is not yet picked up for processing. It can be cancelled. This is also a terminal state.

```typescript
let task = await sc.cancelTask("4b8d323c-19ab-470f-b7c8-d0380b91ca3a");
```

There are other functions like `yieldTask`, `getTask`, `getTasks`, `leaseTask` and `abortTask`. That has been part of library and documentation for those will be added soon.