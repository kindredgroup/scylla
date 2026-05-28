import test from 'ava';

import {v4 as uuid} from 'uuid';
import Scylla, {AddTaskModel, Task, TaskBatch, TaskStatus} from "../index.js";

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

});

(() => {
  const testCases = [
    {
      description: 'addTaskModels is null',
      taskModels: null,
      expectedInsertedTaskRns: [],
      expectedFailedToInsertTaskRns: [],
      expectedInvalidSpecs: [],
    },
    {
      description: 'addTaskModels is undefined',
      taskModels: undefined,
      expectedInsertedTaskRns: [],
      expectedFailedToInsertTaskRns: [],
      expectedInvalidSpecs: [],
    },
    {
      description: 'addTaskModels is empty array',
      taskModels: [],
      expectedInsertedTaskRns: [],
      expectedFailedToInsertTaskRns: [],
      expectedInvalidSpecs: [],
    },
    {
      description: 'spec undefined',
      taskModels: [
        { rnIndex: 0, spec: undefined as any, queue: "single", priority: 0.1 }
      ],
      expectedInsertedTaskRns: [],
      expectedFailedToInsertTaskRns: [],
      expectedInvalidSpecs: [0],
    },
    {
      description: 'spec is null',
      taskModels: [
        {rnIndex: 0, spec: null as any, queue: "single", priority: 0.1 }
      ],
      expectedInsertedTaskRns: [],
      expectedFailedToInsertTaskRns: [],
      expectedInvalidSpecs: [0],
    },
    {
      description: 'spec is empty object',
      taskModels: [{ rnIndex: 0, spec: {}, queue: "single", priority: 0.1 }],
      expectedInsertedTaskRns: [0],
      expectedFailedToInsertTaskRns: [],
      expectedInvalidSpecs: [],
    },
    {
      description: 'first spec is valid and second spec is invalid',
      taskModels: [
        { rnIndex: 0, spec: { job: "1", output: "a" }, queue: "single", priority: 0.1 },
        { rnIndex: 1, spec: null as any, queue: "single", priority: 0.1 },
      ],
      expectedInsertedTaskRns: [0],
      expectedFailedToInsertTaskRns: [],
      expectedInvalidSpecs: [1],
    },
    {
      description: 'both specs are invalid',
      taskModels: [
        { rnIndex: 0, spec: undefined, queue: "single", priority: 0.1 },
        { rnIndex: 1, spec: null, queue: "single", priority: 0.1 },
      ],
      expectedInsertedTaskRns: [],
      expectedFailedToInsertTaskRns: [],
      expectedInvalidSpecs: [0, 1],
    },
    {
      description: 'both specs are valid',
      taskModels: [
        { rnIndex: 0, spec: { job: "1", output: "a" }, queue: "single", priority: 0.1 },
        { rnIndex: 1, spec: { job: "2", output: "b" }, queue: "double", priority: 0.2 },
      ],
      expectedInsertedTaskRns: [0, 1],
      expectedFailedToInsertTaskRns: [],
      expectedInvalidSpecs: [],
    },
    {
      description: 'duplicate specs',
      taskModels: [
        { rnIndex: 0, spec: { job: "1", output: "a" }, queue: "single", priority: 0.1 },
        { rnIndex: 0, spec: { job: "1", output: "a" }, queue: "single", priority: 0.1 },
      ],
      expectedInsertedTaskRns: [0],
      expectedFailedToInsertTaskRns: [],
      expectedInvalidSpecs: [],
    },
  ] satisfies {
    description: string;
    taskModels: (Omit<AddTaskModel, 'rn'> & { rnIndex: number })[] | null | undefined;
    expectedInsertedTaskRns: number[] | undefined;
    expectedFailedToInsertTaskRns: number[] | undefined;
    expectedInvalidSpecs: number[] | undefined;
  }[];

  for (const {
    description,
    taskModels,
    expectedInsertedTaskRns = Array<number>(),
    expectedFailedToInsertTaskRns = Array<number>(),
    expectedInvalidSpecs = Array<number>(),
  } of testCases) {
    test(`add tasks - return correct task batch: ${description}`, async (t) => {
      const uuids = {} as Record<number, string>;
      const getUuidForIndex = (index: number) => {
        if (!uuids[index]) {
          uuids[index] = uuid();
        }

        return uuids[index];
      };

      let sc = await get_singleton_manager();
      let taskBatch = await sc.addTasks(taskModels ? taskModels.map((t) => ({ ...t, rn: getUuidForIndex(t.rnIndex) })) : taskModels as any);
      t.deepEqual(
          taskBatch.inserted.map((t) => t.rn).sort((a, b) => a.localeCompare(b)),
          expectedInsertedTaskRns.map((i) => getUuidForIndex(i)).sort((a, b) => a.localeCompare(b)),
      );
      t.deepEqual(
          taskBatch.failedToInsert.map((t) => t.rn).sort((a, b) => a.localeCompare(b)),
          expectedFailedToInsertTaskRns.map((i) => getUuidForIndex(i)).sort((a, b) => a.localeCompare(b)),
      );
      t.deepEqual(
          taskBatch.invalidSpecs.sort((a, b) => a.localeCompare(b)),
          expectedInvalidSpecs.map((i) => getUuidForIndex(i)).sort((a, b) => a.localeCompare(b)),
      );
    });
  }
})();

test("add tasks - returns correct task batches on repeated calls", async (t) => {
  let sc = await get_singleton_manager();

  let taskToAdd1: AddTaskModel = {
    rn: uuid(),
    spec: {job: "1", output: "a"},
    queue: "single",
    priority: 0.1
  };

  let taskToAdd2: AddTaskModel = {
    rn: uuid(),
    spec: {job: "2", output: "b"},
    queue: "single",
    priority: 0.2
  };
  let taskToAdd3: AddTaskModel = {
    rn: uuid(),
    spec: {job: "3", output: "c"},
    queue: "single",
    priority: 0.3
  };
  let taskToAdd4: AddTaskModel = {
    rn: uuid(),
    spec: {job: "4", output: "d"},
    queue: "single",
    priority: 0.1
  };

  const verifyTaskBatch = async (taskModels: AddTaskModel[], expectedInsertedTaskRns: string[], expectedFailedToInsertTaskRns: string[]) => {
    let taskBatch: TaskBatch = await sc.addTasks(taskModels);
    t.is(taskBatch.inserted.length, expectedInsertedTaskRns.length);
    t.deepEqual(
        taskBatch.inserted.map((t) => t.rn).sort((a, b) => a.localeCompare(b)),
        expectedInsertedTaskRns.sort((a, b) => a.localeCompare(b)),
    );
    t.is(taskBatch.failedToInsert.length, expectedFailedToInsertTaskRns.length);
    t.deepEqual(
        taskBatch.failedToInsert.map((t) => t.rn).sort((a, b) => a.localeCompare(b)),
        expectedFailedToInsertTaskRns.sort((a, b) => a.localeCompare(b)),
    );
  }

  await verifyTaskBatch(
    [taskToAdd1, taskToAdd2, taskToAdd3],
    [taskToAdd1.rn, taskToAdd2.rn, taskToAdd3.rn],
    [],
  );
  await verifyTaskBatch(
      [taskToAdd1, taskToAdd2, taskToAdd3, taskToAdd4],
      [taskToAdd4.rn],
      [taskToAdd1.rn, taskToAdd2.rn, taskToAdd3.rn],
  );
  await verifyTaskBatch(
      [taskToAdd1, taskToAdd2, taskToAdd3, taskToAdd4],
      [],
      [taskToAdd1.rn, taskToAdd2.rn, taskToAdd3.rn, taskToAdd4.rn],
  );
  await verifyTaskBatch([], [], []);
})
