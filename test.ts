import { readFile, readDir, writeFile } from "fs";
import { stdout } from "os";
import { pipe } from "pipe";
import { createConsole } from "util";

const decoder = new TextDecoder();
const encoder = new TextEncoder();

// let KEY = 0;
// function uniqueKey() {
//   return KEY++;
// }

// class Queue {
//   tasks: Map<number, Promise<unknown>> = new Map();

//   push<T>(task: Promise<T>): Promise<T> {
//     let id = uniqueKey();
//     this.tasks.set(id, task);

//     return task.then(
//       (resp) => {
//         this.tasks.delete(id);
//         return resp;
//       },
//       (err) => {
//         this.tasks.delete(id);
//         // delete this.tasks[id];
//         throw err;
//       }
//     );
//   }

//   async wait() {
//     // print("Vent " + this.tasks.values().length);
//     await delay(60);
//     await Promise.all(Array.from(this.tasks.values()));
//   }
// }

// const QUEUE = new Queue();

// const console = createConsole((arg) => {
//   const p = stdout.write(encoder.encode(arg));
//   QUEUE.push(p);
// });

export async function main(greet) {
  console.log("Started", greet);
  const lines = await pipe(readDir("types"))
    .filter((m) => !m.path.endsWith("index.d.ts"))
    .map((m) => readFile(m.path))
    .map((m) => decoder.decode(m))

    .join("\n\n");

  // console.log(lines);

  await writeFile("scriptor.d.ts", lines);
  // await delay(1000);
  // console.log("done");

  for (let i = 0; i < 100; i++) {
    console.log("TEST ");
    // delay(10);
  }
  console.log("done");
}
