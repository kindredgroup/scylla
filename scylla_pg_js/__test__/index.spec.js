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
    pgUser: "admin",
    pgPassword: "admin",
    pgDatabase: "scylla"
  })
  root_sc = sc;
  return root_sc;
}

test("should be able to add a new task", async (t) => {
  let sc = get_singleton_manager();
  let task_to_added = {
    rn: uuid(),
    spec: JSON.stringify({job: "1", output: "f"}),
    queue: "single",
    priority: 0.1
  };
  let task_added = JSON.parse(await sc.addTask(task_to_added));
  t.is(task_added.rn, task_to_added.rn);

})
