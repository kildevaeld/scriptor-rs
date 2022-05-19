import { createConsole } from "util";
import { stdout } from "os";
import { enqueueTask, awaitAllTasks } from "tasks";

const encoder = new TextEncoder();

globalThis.console = createConsole((arg) => {
  const p = stdout.write(encoder.encode(arg));
  enqueueTask(p);
});

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
