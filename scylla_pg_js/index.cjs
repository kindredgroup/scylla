// @ts-ignore
const rustLib = require('./scylla.node');

class Scylla {
  scyllaRust;

  async initiate(dbConfig) {
    let resp = await rustLib.ScyllaManager.initPgConfig(dbConfig);
    this.scyllaRust = resp;
  }

  async getTask(rn) {
    let response = await this.scyllaRust.getTask(rn);
    return JSON.parse(response);
  }

  async getTasks(getTaskModel = {}) {
    let response = await this.scyllaRust.getTasks(getTaskModel);
    return JSON.parse(response);
  }

  async addTask(addTaskModel) {
    let response = await this.scyllaRust.addTask(addTaskModel);
    return JSON.parse(response);
  }
  
  async leaseTask(rn, worker) {
    let response = await this.scyllaRust.leaseTask(rn, worker);
    return JSON.parse(response);
  }

  async heartBeatTask(rn, progress) {
    let response = await this.scyllaRust.heartBeatTask(rn, progress);
    return JSON.parse(response);
  }

  async cancelTask(rn) {
    let response = await this.scyllaRust.cancelTask(rn);
    return JSON.parse(response);
  }

  async completeTask(rn) {
    let response = await this.scyllaRust.completeTask(rn);
    return JSON.parse(response);
  }

  async yieldTask(rn) {
    let response = await this.scyllaRust.yieldTask(rn);
    return JSON.parse(response);
  }

  async abortTask(rn, taskError) {
    if (!taskError || !taskError.args)
      throw Error ( "Invalid argument. Object cannot be undefined" );
    let response = JSON.parse(await this.scyllaRust.abortTask(rn, {...taskError, args: JSON.stringify(taskError.args)}));
    return JSON.parse(response);
  }
}

module.exports = Scylla
