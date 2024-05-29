import { ScyllaManager } from "scylla_pg_js";
export var TaskStatus;
(function (TaskStatus) {
    TaskStatus["running"] = "running";
    TaskStatus["ready"] = "ready";
    TaskStatus["cancelled"] = "cancelled";
    TaskStatus["completed"] = "completed";
    TaskStatus["aborted"] = "aborted";
})(TaskStatus || (TaskStatus = {}));
export var TaskHistoryType;
(function (TaskHistoryType) {
    TaskHistoryType["assignment"] = "TaskAssignment";
    TaskHistoryType["yield"] = "TaskYield";
    TaskHistoryType["timeout"] = "TaskTimeout";
})(TaskHistoryType || (TaskHistoryType = {}));
class Scylla {
    constructor(sc) {
        this.scyllaManager = sc;
    }
    static async initiate(dbConfig) {
        let scyllaManager = await ScyllaManager.initPgConfig(dbConfig);
        console.log("[SCYLA] dbConfig...", dbConfig);
        // console.log("[SCYLA] Scylla manager...", await scyllaManager.getTasks({ limit: 20 }))
        let sc = new Scylla(scyllaManager);
        return sc;
    }
    async getTask(rn) {
        let resp = await this.scyllaManager.getTask(rn);
        return JSON.parse(resp);
    }
    async getTasks(getTaskModel = {}) {
        let resp = await this.scyllaManager.getTasks(getTaskModel);
        return JSON.parse(resp);
    }
    async addTask(addTaskModel) {
        if (!addTaskModel || !addTaskModel.spec) {
            throw Error("Invalid argument. addTaskModel.spec cannot be undefined");
        }
        let atm = {
            ...addTaskModel,
            spec: JSON.stringify(addTaskModel.spec),
        };
        let response = "";
        try {
            response = await this.scyllaManager.addTask(atm);
        }
        catch (error) {
            console.error("[SCYLA] Error adding task...", error);
        }
        return JSON.parse(response);
    }
    async leaseTask(rn, worker, taskTimeOutInSecs) {
        let response = await this.scyllaManager.leaseTask(rn, worker, taskTimeOutInSecs);
        return JSON.parse(response);
    }
    async leaseNTasks(queue, limit, worker, taskTimeOutInSecs) {
        let response = await this.scyllaManager.leaseNTasks(queue, limit, worker, taskTimeOutInSecs);
        return JSON.parse(response);
    }
    async heartBeatTask(rn, worker, progress, taskTimeOutInSecs) {
        let response = await this.scyllaManager.heartBeatTask(rn, worker, progress, taskTimeOutInSecs);
        return JSON.parse(response);
    }
    async cancelTask(rn) {
        let response = await this.scyllaManager.cancelTask(rn);
        return JSON.parse(response);
    }
    async completeTask(rn) {
        let response = await this.scyllaManager.completeTask(rn);
        return JSON.parse(response);
    }
    async yieldTask(rn) {
        let response = await this.scyllaManager.yieldTask(rn);
        return JSON.parse(response);
    }
    async abortTask(rn, taskError) {
        if (!taskError || !taskError.args) {
            throw Error("Invalid argument. taskError.args cannot be undefined");
        }
        let response = await this.scyllaManager.abortTask(rn, { ...taskError, args: JSON.stringify(taskError.args) });
        return JSON.parse(response);
    }
}
export default Scylla;
