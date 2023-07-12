import test from 'ava';
// const snappy = require("snappy");
//
// test("snappy test", async (t) => {
//   let compressedString = await snappy.compress("test");
//   console.log(compressedString);
//   t.assert("test" == compressedString);
// })
import {v4 as uuid} from 'uuid';
import ScyllaManager, {AddTaskModel} from "../index.js";

let root_sc: ScyllaManager | null = null;
async function get_singleton_manager() {
 if (!!root_sc) {
   return root_sc
 }

 let sc = await ScyllaManager.initiate({
   pgHost: "127.0.0.1",
   pgPort: 5432,
   pgUser: "admin",
   pgPassword: "admin",
   pgDatabase: "scylla"
 });
 root_sc = sc;
 return root_sc;
}

test("throws in case of failed connection", async (t) => {
  try {
    let sc = await get_singleton_manager();
    let task_to_added: AddTaskModel = {
      rn: uuid(),
      spec: {job: "1", output: "f"},
      queue: "single",
      priority: 0.1
    };
    await t.throwsAsync(async () => {return sc.addTask(task_to_added)}, {
      code: "GenericFailure"
    });
//  let task_added = JSON.parse(await sc.addTask(task_to_added));

//  t.is(task_added.rn, task_to_added.rn);
  } catch (e) {
    console.log("excecption: ", e);
  }

})
