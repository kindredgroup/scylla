import test from 'ava';

import {v4 as uuid} from 'uuid';
import Scylla, {AddTaskModel, Task, TaskStatus} from "../index.js";

let root_sc: Scylla | null = null;
async function get_singleton_manager() {
 if (!!root_sc) {
   return root_sc
 }

 let sc = await Scylla.initiate({
     pgHost: process.env["PG_HOST"]!,
     pgPort: parseInt(process.env["PG_PORT"]!),
     pgUser: process.env["PG_USER"]!,
     pgPassword: process.env["PG_PASSWORD"]!,
     pgDatabase: process.env["PG_DATABASE"]!,
     pgPoolSize: parseInt(process.env["PG_POOL_SIZE"]!),
 });
 root_sc = sc;
 return root_sc;
}

test("add, lease and yield", async (t) => {
  let sc = await get_singleton_manager();
  let taskToAdd: AddTaskModel = {
    rn: uuid(),
    spec: {job: "1", output: "f"},
    queue: "single",
    priority: 0.1
  };

 let taskAdded: Task = await sc.addTask(taskToAdd);

 t.is(taskAdded.rn, taskToAdd.rn);
 let leasedTask = await sc.leaseTask(taskAdded.rn, "worker");
 t.is(leasedTask.status, TaskStatus.running);
 let heartBeatTask = await sc.heartBeatTask(taskAdded.rn, "worker", 0.2);
 t.is(heartBeatTask.status, TaskStatus.running);
 t.is(heartBeatTask.progress, 0.2);
 try {
     await sc.heartBeatTask(taskAdded.rn, "worker1", 0.3); // Only owner can extend the heartbeat
     t.is(false, true); // if last step ran without error. test should fail
 } catch (e: any) {
     t.is(e.message, "Validation failed: Only owner can extend the heartbeat.")
 }
 let yieldedTask = await sc.yieldTask(taskAdded.rn);
  t.is(yieldedTask.status, TaskStatus.running); // this needs monitor to run in parallel
})

test("add and cancel", async (t) => {
  let sc = await get_singleton_manager();
  let taskToAdd = {
    rn: uuid(),
    spec: {job: "1", output: "f"},
    queue: "single",
    priority: 0.1
  };

  let taskAdded: Task =await sc.addTask(taskToAdd);

  t.is(taskAdded.rn, taskToAdd.rn);
  let taskCancelled = await sc.cancelTask(taskToAdd.rn);
  t.is(taskCancelled.status, TaskStatus.cancelled);
})

test("add, lease with timeout and complete", async (t) => {
  let sc = await get_singleton_manager();
  let taskToAdd = {
    rn: uuid(),
    spec: {job: "1", output: "f"},
    queue: "single",
    priority: 0.1
  };

  let taskAdded = await sc.addTask(taskToAdd);

  t.is(taskAdded.rn, taskToAdd.rn);
  let leasedTask = await sc.leaseTask(taskAdded.rn, "worker", 20);
  t.is(leasedTask.status, TaskStatus.running);
  let completedTask = await sc.completeTask(taskAdded.rn);
  t.is(completedTask.status, TaskStatus.completed);
})

test("add, lease N Tasks with timeout, complete and abort", async (t) => {
  let sc = await get_singleton_manager();
  let taskToAdd1 = {
    rn: uuid(),
    spec: {job: "1", output: "f"},
    queue: "single",
    priority: 100
  };
  let taskToAdd2 = {
    rn: uuid(),
    spec: {job: "1", output: "f"},
    queue: "single",
    priority: 120
  };
  let taskToAdd3 = {
    rn: uuid(),
    spec: {job: "1", output: "f"},
    queue: "single",
    priority: 127
  };

  let taskAdded1 = await sc.addTask(taskToAdd1);
  let taskAdded2 = await sc.addTask(taskToAdd2);
  let taskAdded3 = await sc.addTask(taskToAdd3);
  t.is(taskAdded1.rn, taskToAdd1.rn);
  t.is(taskAdded2.rn, taskToAdd2.rn);
  t.is(taskAdded3.rn, taskToAdd3.rn);
  t.is(taskAdded1.status, TaskStatus.ready);
  t.is(taskAdded2.status, TaskStatus.ready);
  t.is(taskAdded3.status, TaskStatus.ready);
  let leasedTask = await sc.leaseNTasks("single", 2, "worker", 20);
  t.is(leasedTask.length, 2);
  t.is(leasedTask[0].status, TaskStatus.running);
  t.is(leasedTask[1].status, TaskStatus.running);
  let completedTask = await sc.completeTask(leasedTask[1].rn);

  let abortedTask = await sc.abortTask(leasedTask[0].rn, {
    code: "UNIT TEST",
    args: {"key": "value"},
    description: "UNIT TEST ABORTED"
  });
  t.is(abortedTask.status, TaskStatus.aborted);

  t.is(completedTask.status, TaskStatus.completed);

})


