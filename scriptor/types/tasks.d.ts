declare module "tasks" {
  export function enqueueTask<T>(task: Promise<T>): Promise<T>;

  export function awaitAllTasks(): Promise<void>;
}
