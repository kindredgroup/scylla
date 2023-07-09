// import Scylla, {TaskStatus} from "../scylla_pg_client";

import Scylla,{TaskStatus} from "scylla_pg_client";

(async () => {
  console.log("type of scylla", typeof Scylla);
  let sc = await Scylla.initiate(
    {
      pgHost: "127.0.0.1",
      pgPort: 5432,
      pgUser: "admin",
      pgPassword: "admin",
      pgDatabase: "scylla"
    });

  let atm = {
    rn: "sawd",
    queue: "long_queue",
    priority: 0.4,
    spec: {a: 1, b: 2}
  }
  let task_added = await sc.addTask(atm);
  console.log("task added", task_added);
  let task = await sc.getTask(atm.rn);
  console.log("task fetched", task);
  let running_tasks = await sc.getTasks({ status: TaskStatus.running});
  console.log("running_tasks fetched", running_tasks);
})();
