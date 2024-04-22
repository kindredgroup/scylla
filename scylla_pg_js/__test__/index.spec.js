const test = require("ava");
const {v4: uuid} = require('uuid');
const {ScyllaManager} = require('../index.js');

let root_sc = null;

function get_singleton_manager() {
    if (!!root_sc) {
        return root_sc
    }

    let sc = ScyllaManager.initPgConfig({
        pgHost: process.env["PG_HOST"],
        pgPort: parseInt(process.env["PG_PORT"]),
        pgUser: process.env["PG_USER"],
        pgPassword: process.env["PG_PASSWORD"],
        pgDatabase: process.env["PG_DATABASE"],
        pgPoolSize: parseInt(process.env["PG_POOL_SIZE"]),
    })
    root_sc = sc;
    return root_sc;
}

test("add, lease and yield", async (t) => {
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
    let heartBeatTask = JSON.parse(await sc.heartBeatTask(taskAdded.rn, "worker", 0.2));
    t.is(heartBeatTask.status, "running");
    t.is(heartBeatTask.progress, 0.2);
    try {
        await sc.heartBeatTask(taskAdded.rn, "worker1", 0.3);
        t.is(false, true); // if last step ran without error. test should fail
    } catch (e) {
        t.is(e.message, "Validation failed: Only owner can extend the heartbeat.")
    }
    let yieldedTask = JSON.parse(await sc.yieldTask(taskAdded.rn));
    t.is(yieldedTask.status, "running"); // this needs monitor to run in parallel
})

test("add and cancel", async (t) => {
    let sc = get_singleton_manager();
    let taskToAdd = {
        rn: uuid(),
        spec: JSON.stringify({job: "1", output: "f"}),
        queue: "single",
        priority: 0.1
    };

    let taskAdded = JSON.parse(await sc.addTask(taskToAdd));

    t.is(taskAdded.rn, taskToAdd.rn);
    let taskCancelled = JSON.parse(await sc.cancelTask(taskToAdd.rn));
    t.is(taskCancelled.status, "cancelled");
})

test("add, lease with timeout and complete", async (t) => {
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
    let completedTask = JSON.parse(await sc.completeTask(taskAdded.rn));
    t.is(completedTask.status, "completed");
})

test("add, lease N Tasks with timeout, complete and abort", async (t) => {
    let sc = get_singleton_manager();
    let taskToAdd1 = {
        rn: uuid(),
        spec: JSON.stringify({job: "1", output: "f"}),
        queue: "single",
        priority: 0.1
    };
    let taskToAdd2 = {
        rn: uuid(),
        spec: JSON.stringify({job: "1", output: "f"}),
        queue: "single",
        priority: 0.1
    };

    let taskAdded1 = JSON.parse(await sc.addTask(taskToAdd1));
    let taskAdded2 = JSON.parse(await sc.addTask(taskToAdd2));

    t.is(taskAdded1.rn, taskToAdd1.rn);
    t.is(taskAdded2.rn, taskToAdd2.rn);
    let leasedTask = JSON.parse(await sc.leaseNTasks("single", 2, "worker", 20));
    t.is(leasedTask.length, 2);
    t.is(leasedTask[0].status, "running");
    t.is(leasedTask[1].status, "running");
    let completedTask = JSON.parse(await sc.completeTask(leasedTask[1].rn));
    let abortedTask = JSON.parse(await sc.abortTask(leasedTask[0].rn, {
        code: "UNIT TEST",
        args: JSON.stringify({"key": "value"}),
        description: "UNIT TEST ABORTED"
    }));
    t.is(completedTask.status, "completed");
    t.is(abortedTask.status, "aborted");
})

