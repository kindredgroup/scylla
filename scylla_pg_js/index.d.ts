export declare enum TaskStatus {
  running="running",
  ready="ready",
  cancelled="cancelled",
  completed="completed",
  aborted="aborted"
}

export declare enum TaskHistoryType {
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
  worker?: object
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
  host: string
  port: number
  user: string
  password: string
  dbName: string
};


export declare const initPgConfig: (dbConfig: DbConfig) => Promise<void>;

export declare const addTask: (addTaskModel: AddTaskModel) => Promise<Task>;

export declare const leaseTask: (rn: string, worker: string) => Promise<Task>;

export declare const yieldTask: (rn: string) => Promise<Task>;

export declare const cancelTask: (rn: string) => Promise<Task>;

export declare const completeTask: (rn: string) => Promise<Task>;

export declare const abortTask: (rn: string, taskError: TaskError) => Promise<Task>;

export declare const heartBeatTask: (rn: string, progress?: number) => Promise<Task>;

export declare const getTask: (rn: string) => Promise<Task>;

export declare const getTasks: (rn: GetTaskModel) => Promise<Task[]>;
