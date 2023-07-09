"use strict";
var __awaiter = (this && this.__awaiter) || function (thisArg, _arguments, P, generator) {
    function adopt(value) { return value instanceof P ? value : new P(function (resolve) { resolve(value); }); }
    return new (P || (P = Promise))(function (resolve, reject) {
        function fulfilled(value) { try { step(generator.next(value)); } catch (e) { reject(e); } }
        function rejected(value) { try { step(generator["throw"](value)); } catch (e) { reject(e); } }
        function step(result) { result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected); }
        step((generator = generator.apply(thisArg, _arguments || [])).next());
    });
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.TaskHistoryType = exports.TaskStatus = void 0;
const scylla_pg_js_1 = require("scylla_pg_js");
var TaskStatus;
(function (TaskStatus) {
    TaskStatus["running"] = "running";
    TaskStatus["ready"] = "ready";
    TaskStatus["cancelled"] = "cancelled";
    TaskStatus["completed"] = "completed";
    TaskStatus["aborted"] = "aborted";
})(TaskStatus = exports.TaskStatus || (exports.TaskStatus = {}));
var TaskHistoryType;
(function (TaskHistoryType) {
    TaskHistoryType["assignment"] = "TaskAssignment";
    TaskHistoryType["yield"] = "TaskYield";
    TaskHistoryType["timeout"] = "TaskTimeout";
})(TaskHistoryType = exports.TaskHistoryType || (exports.TaskHistoryType = {}));
class Scylla {
    constructor(sc) {
        this.scyllaManager = sc;
    }
    static initiate(dbConfig) {
        return __awaiter(this, void 0, void 0, function* () {
            let scyllaManager = yield scylla_pg_js_1.ScyllaManager.initPgConfig(dbConfig);
            let sc = new Scylla(scyllaManager);
            return sc;
        });
    }
    getTask(rn) {
        return __awaiter(this, void 0, void 0, function* () {
            let resp = yield this.scyllaManager.getTask(rn);
            return JSON.parse(resp);
        });
    }
    getTasks(getTaskModel = {}) {
        return __awaiter(this, void 0, void 0, function* () {
            let resp = yield this.scyllaManager.getTasks(getTaskModel);
            return JSON.parse(resp);
        });
    }
    addTask(addTaskModel) {
        return __awaiter(this, void 0, void 0, function* () {
            if (!addTaskModel || !addTaskModel.spec) {
                throw Error("Invalid argument. addTaskModel.spec cannot be undefined");
            }
            let atm = Object.assign(Object.assign({}, addTaskModel), { spec: JSON.stringify(addTaskModel.spec) });
            let response = yield this.scyllaManager.addTask(atm);
            return JSON.parse(response);
        });
    }
    leaseTask(rn, worker) {
        return __awaiter(this, void 0, void 0, function* () {
            let response = yield this.scyllaManager.leaseTask(rn, worker);
            return JSON.parse(response);
        });
    }
    heartBeatTask(rn, progress) {
        return __awaiter(this, void 0, void 0, function* () {
            let response = yield this.scyllaManager.heartBeatTask(rn, progress);
            return JSON.parse(response);
        });
    }
    cancelTask(rn) {
        return __awaiter(this, void 0, void 0, function* () {
            let response = yield this.scyllaManager.cancelTask(rn);
            return JSON.parse(response);
        });
    }
    completeTask(rn) {
        return __awaiter(this, void 0, void 0, function* () {
            let response = yield this.scyllaManager.completeTask(rn);
            return JSON.parse(response);
        });
    }
    yieldTask(rn) {
        return __awaiter(this, void 0, void 0, function* () {
            let response = yield this.scyllaManager.yieldTask(rn);
            return JSON.parse(response);
        });
    }
    abortTask(rn, taskError) {
        return __awaiter(this, void 0, void 0, function* () {
            if (!taskError || !taskError.args) {
                throw Error("Invalid argument. taskError.args cannot be undefined");
            }
            let response = JSON.parse(yield this.scyllaManager.abortTask(rn, Object.assign(Object.assign({}, taskError), { args: JSON.stringify(taskError.args) })));
            return JSON.parse(response);
        });
    }
}
exports.default = Scylla;
