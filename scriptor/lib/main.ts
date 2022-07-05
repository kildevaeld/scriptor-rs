import { createConsole } from "util";
import { enqueueTask, awaitAllTasks } from "tasks";
import { stdout, stderr } from "os";

const encoder = new TextEncoder();

globalThis.console = createConsole(
  (arg) => {
    enqueueTask(stdout.write(encoder.encode(arg)));
  },
  (arg) => {
    enqueueTask(stderr.write(encoder.encode(arg)));
  }
);

export async function main(path: string, arg: unknown) {
  const module = await import(path);

  if (!module) {
    throw new Error("module is undefined");
  }

  try {
    if (typeof module.default === "function") {
      await module.default(arg);
    } else if (typeof module.main === "function") {
      await module.main(arg);
    }
    await awaitAllTasks();
  } catch (e) {
    await awaitAllTasks();
    throw e;
  }
}
