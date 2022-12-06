import Scylla from './index.cjs';
import {uuid} from 'uuidv4';

(async () => {
  let sc = new Scylla();
  await sc.initiate({host: "127.0.0.1", port: 5432, user: "admin", password: "admin", dbName: "scylla"});
  
  let atm = {
    rn: "1234",//uuid(),
    queue: "long_queue",
    priority: 0.4,
    spec: JSON.stringify({a: 1, b: 2})
  }
  let task_added = await sc.addTask(atm);
  console.log(task_added);
  let task = await sc.getTask(atm.rn);
  console.log(task);
})();