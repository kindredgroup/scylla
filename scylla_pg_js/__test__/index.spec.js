const test = require("ava");
const { v4: uuid } = require('uuid');
const { ScyllaManager } = require('../index.js');

let root_sc = null;
function get_singleton_manager() {
  if (!!root_sc) {
    return root_sc
  }

  let sc = ScyllaManager.initPgConfig({
    pgHost: "127.0.0.1",
    pgPort: 5432,
    pgUser: "postgres",
    pgPassword: "admin",
    pgDatabase: "scylla"
  })
  root_sc = sc;
  return root_sc;
}

test("add and lease", async (t) => {
  let sc = get_singleton_manager();
  let taskToAdd = {
    rn: uuid(),
    spec: JSON.stringify({job: "1", output: "f"}),
    queue: "single",
    priority: 0.1
  };

 let taskAdded = JSON.parse(await sc.addTask(taskToAdd));

 t.is(taskAdded.rn, taskToAdd.rn);
 let leasedTask = JSON.parse(await sc.leaseTask(taskAdded.rn, "worker"));
 t.is(leasedTask.status, "running");
})

test("add and lease with timeout", async (t) => {
  let sc = get_singleton_manager();
  let taskToAdd = {
    rn: uuid(),
    spec: JSON.stringify({job: "1", output: "f"}),
    queue: "single",
    priority: 0.1
  };

  let taskAdded = JSON.parse(await sc.addTask(taskToAdd));

  t.is(taskAdded.rn, taskToAdd.rn);
  let leasedTask = JSON.parse(await sc.leaseTask(taskAdded.rn, "worker", 20));
  t.is(leasedTask.status, "running");
})

