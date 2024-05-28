export declare enum TaskStatus {
    running = "running",
    ready = "ready",
    cancelled = "cancelled",
    completed = "completed",
    aborted = "aborted"
}
export declare enum TaskHistoryType {
    assignment = "TaskAssignment",
    yield = "TaskYield",
    timeout = "TaskTimeout"
}
export declare type TaskHistory = {
    typ: TaskHistoryType;
    worker: string;
    time: string;
    progress?: number;
};
export declare type TaskError = {
    code: string;
    args: object;
    description: string;
};
export declare type AddTaskModel = {
    rn: string;
    queue: string;
    spec: object;
    priority: number;
};
export declare type GetTaskModel = {
    status?: TaskStatus;
    queue?: string;
    worker?: string;
    limit?: number;
};
export declare type Task = {
    rn: string;
    queue: string;
    spec: object;
    priority: number;
    status: TaskStatus;
    progress: number;
    created: string;
    updated: string;
    deadline: string;
    owner: string;
    errors: TaskError[];
    history: TaskHistory[];
};
export declare type DbConfig = {
    pgHost: string;
    pgPort: number;
    pgUser: string;
    pgPassword: string;
    pgDatabase: string;
    pgPoolSize: number;
};
declare class Scylla {
    private scyllaManager;
    private constructor();
    static initiate(dbConfig: DbConfig): Promise<Scylla>;
    getTask(rn: string): Promise<Task>;
    getTasks(getTaskModel?: GetTaskModel): Promise<Task[]>;
    addTask(addTaskModel: AddTaskModel): Promise<Task>;
    leaseTask(rn: string, worker: string, taskTimeOutInSecs?: number): Promise<Task>;
    leaseNTasks(queue: string, limit: number, worker: string, taskTimeOutInSecs?: number): Promise<Task[]>;
    heartBeatTask(rn: string, worker: string, progress?: number, taskTimeOutInSecs?: number): Promise<Task>;
    cancelTask(rn: string): Promise<Task>;
    completeTask(rn: string): Promise<Task>;
    yieldTask(rn: string): Promise<Task>;
    abortTask(rn: string, taskError: TaskError): Promise<Task>;
}
export default Scylla;
