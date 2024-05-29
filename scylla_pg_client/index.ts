import { JsAddTaskModel, JsDbConfig, JsGetTasksModel, ScyllaManager } from "scylla_pg_js";


export enum TaskStatus {
  running = "running",
  ready = "ready",
  cancelled = "cancelled",
  completed = "completed",
  aborted = "aborted"
}

export enum TaskHistoryType {
  assignment = "TaskAssignment",
  yield = "TaskYield",
  timeout = "TaskTimeout"
}
export declare type TaskHistory = {
  typ: TaskHistoryType
  worker: string
  time: string
  progress?: number
}

export declare type TaskError = {
  code: string
  args: object
  description: string
}

export declare type AddTaskModel = {
  rn: string
  queue: string
  spec: object
  priority: number
};

export declare type GetTaskModel = {
  status?: TaskStatus
  queue?: string
  worker?: string
  limit?: number
};

export declare type Task = {
  rn: string
  queue: string
  spec: object
  priority: number
  status: TaskStatus
  progress: number
  created: string
  updated: string
  deadline: string
  owner: string
  errors: TaskError[]
  history: TaskHistory[]
};

export declare type DbConfig = {
  pgHost: string
  pgPort: number
  pgUser: string
  pgPassword: string
  pgDatabase: string
  pgPoolSize: number
};

class Scylla {
  private scyllaManager: ScyllaManager;
  private constructor(sc: ScyllaManager) {
    this.scyllaManager = sc
  }
  public static async initiate(dbConfig: DbConfig): Promise<Scylla> {
    let scyllaManager: ScyllaManager = await ScyllaManager.initPgConfig(dbConfig as JsDbConfig)
    console.log("[SCYLA] dbConfig...", dbConfig)
    // console.log("[SCYLA] Scylla manager...", await scyllaManager.getTasks({ limit: 20 }))
    let sc = new Scylla(scyllaManager);
    return sc;
  }
  public async getTask(rn: string): Promise<Task> {
    let resp = await this.scyllaManager.getTask(rn);
    return JSON.parse(resp);
  }
  public async getTasks(getTaskModel: GetTaskModel = {}): Promise<Task[]> {
    let resp = await this.scyllaManager.getTasks(getTaskModel as JsGetTasksModel);
    return JSON.parse(resp);
  }
  public async addTask(addTaskModel: AddTaskModel): Promise<Task> {
    if (!addTaskModel || !addTaskModel.spec) {
      throw Error("Invalid argument. addTaskModel.spec cannot be undefined");
    }
    let atm: JsAddTaskModel = {
      ...addTaskModel,
      spec: JSON.stringify(addTaskModel.spec),
    }

    let response: string = ""
    try {
      response = await this.scyllaManager.addTask(atm);

    } catch (error) {
      console.error("[SCYLA] Error adding task...", error)
    }
    return JSON.parse(response);
  }

  public async leaseTask(rn: string, worker: string, taskTimeOutInSecs?: number): Promise<Task> {
    let response = await this.scyllaManager.leaseTask(rn, worker, taskTimeOutInSecs);
    return JSON.parse(response);
  }

  public async leaseNTasks(queue: string, limit: number, worker: string, taskTimeOutInSecs?: number): Promise<Task[]> {
    let response = await this.scyllaManager.leaseNTasks(queue, limit, worker, taskTimeOutInSecs);
    return JSON.parse(response);
  }

  public async heartBeatTask(rn: string, worker: string, progress?: number, taskTimeOutInSecs?: number): Promise<Task> {
    let response = await this.scyllaManager.heartBeatTask(rn, worker, progress, taskTimeOutInSecs);
    return JSON.parse(response);
  }

  public async cancelTask(rn: string): Promise<Task> {
    let response = await this.scyllaManager.cancelTask(rn);
    return JSON.parse(response);
  }

  public async completeTask(rn: string): Promise<Task> {
    let response = await this.scyllaManager.completeTask(rn);
    return JSON.parse(response);
  }

  public async yieldTask(rn: string): Promise<Task> {
    let response = await this.scyllaManager.yieldTask(rn);
    return JSON.parse(response);
  }

  public async abortTask(rn: string, taskError: TaskError): Promise<Task> {
    if (!taskError || !taskError.args) {
      throw Error("Invalid argument. taskError.args cannot be undefined");
    }
    let response = await this.scyllaManager.abortTask(rn, { ...taskError, args: JSON.stringify(taskError.args) });
    return JSON.parse(response);
  }
}
export default Scylla;
