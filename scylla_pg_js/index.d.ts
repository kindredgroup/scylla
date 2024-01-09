/* tslint:disable */
/* eslint-disable */

/* auto-generated by NAPI-RS */

export interface JsAddTaskModel {
  rn: string
  spec: string
  priority: number
  queue: string
}
export interface JsGetTasksModel {
  worker?: string
  status?: string
  limit?: number
  queue?: string
}
export interface JsTaskError {
  code: string
  args: string
  description: string
}
export interface JsDbConfig {
  pgHost: string
  pgPort: number
  pgUser: string
  pgPassword: string
  pgDatabase: string
}
export class ScyllaManager {
  /**
   * # Errors
   * Convert rust error into `napi::Error`
   */
  static initPgConfig(jsDbConfig: JsDbConfig): ScyllaManager
  /**
   * # Errors
   * Convert rust error into `napi::Error`
   */
  getTask(rn: string): Promise<string>
  /**
   * # Errors
   * Convert rust error into `napi::Error`
   */
  getTasks(jsGtm: JsGetTasksModel): Promise<string>
  /**
   * # Errors
   * Convert rust error into `napi::Error`
   */
  addTask(jsAtm: JsAddTaskModel): Promise<string>
  /**
   * # Errors
   * Convert rust error into `napi::Error`
   */
  leaseTask(rn: string, worker: string, taskTimeoutInSecs?: number | undefined | null): Promise<string>
  leaseNTasks(queue: string, limit: number, worker: string, taskTimeoutInSecs?: number | undefined | null): Promise<string>
  /**
   * # Errors
   * Convert rust error into `napi::Error`
   */
  yieldTask(rn: string): Promise<string>
  /**
   * # Errors
   * Convert rust error into `napi::Error`
   */
  completeTask(rn: string): Promise<string>
  /**
   * # Errors
   * Convert rust error into `napi::Error`
   */
  cancelTask(rn: string): Promise<string>
  /**
   * # Errors
   * Convert rust error into `napi::Error`
   */
  abortTask(rn: string, jsError: JsTaskError): Promise<string>
  /**
   * # Errors
   * Convert rust error into `napi::Error`
   */
  heartBeatTask(rn: string, progress?: number | undefined | null, taskTimeoutInSecs?: number | undefined | null): Promise<string>
}
