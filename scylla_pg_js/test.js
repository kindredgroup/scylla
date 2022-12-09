import Scylla from './index.cjs';
import {uuid} from 'uuidv4';

(async () => {
  let sc = new Scylla();
  await sc.initiate({pgHost: "127.0.0.1", pgPort: 5432, pgUser: "admin", pgPassword: "admin", pgDatabase: "scylla"});
  
  let atm = {
    rn: uuid(),
    queue: "long_queue",
    priority: 0.4,
    spec: JSON.stringify({a: 1, b: 2})
  }
  let task_added = await sc.addTask(atm);
  console.log("task added", task_added);
  let task = await sc.getTask(atm.rn);
  console.log("task fetched", task);
  let running_tasks = await sc.getTasks({ status: "running"});
  console.log("running_tasks fetched", running_tasks);
})();