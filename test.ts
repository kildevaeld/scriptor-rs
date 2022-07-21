import { readFile, readDir, open } from "fs";
import { pipe } from "pipe";

delay(1000).then(() => print("Hello timeout"));

// import greeting from "./import";
// import { createConsole } from "util";

const decoder = new TextDecoder();

export async function main(greet) {
  print("Hello, World!");
  const file = await open("scriptor.d.ts", "cw");
  await pipe(readDir("scriptor/types"))
    .filter((m) => !m.path.endsWith("index.d.ts"))
    .map((m) => readFile(m.path))
    .map((m) => decoder.decode(m))
    .forEach((m) => file.write(m + "\n\n"));
  await file.flush();

  print("Another");
  // console.log({
  //   hello: "world",
  // });
}
