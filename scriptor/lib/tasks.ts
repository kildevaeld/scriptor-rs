let KEY = 0;
function uniqueKey() {
  return KEY++;
}

class Queue {
  tasks: Map<number, Promise<unknown>> = new Map();

  push<T>(task: Promise<T>): Promise<T> {
    let id = uniqueKey();

    const tasks = this.tasks;

    const out = task.then(
      (resp) => {
        tasks.delete(id);
        return resp;
      },
      (err) => {
        tasks.delete(id);
        throw err;
      }
    );

    tasks.set(id, out);

    return out;
  }

  async wait() {
    await delay(16);

    while (this.tasks.size > 0) {
      const tasks = Array.from(this.tasks.values());
      this.tasks = new Map();
      await Promise.all(tasks);
    }
  }
}

const QUEUE = new Queue();

export function enqueueTask<T>(task: Promise<T>): Promise<T> {
  return QUEUE.push(task);
}

export async function awaitAllTasks(): Promise<void> {
  await QUEUE.wait();
}
