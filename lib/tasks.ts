let KEY = 0;
function uniqueKey() {
  return KEY++;
}

class Queue {
  tasks: Map<number, Promise<unknown>> = new Map();

  push<T>(task: Promise<T>): Promise<T> {
    let id = uniqueKey();

    const out = task.then(
      (resp) => {
        this.tasks.delete(id);
        return resp;
      },
      (err) => {
        this.tasks.delete(id);
        throw err;
      }
    );

    this.tasks.set(id, out);

    return out;
  }

  async wait() {
    await delay(60);
    while (this.tasks.size > 0) {
      await Promise.all(Array.from(this.tasks.values()));
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
