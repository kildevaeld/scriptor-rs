import { readFile, readDir, open } from "fs";
import { pipe } from "pipe";

import greeting from "./import";
import { createConsole } from "util";

const decoder = new TextDecoder();

export async function main(greet) {
  const file = await open("scriptor.d.ts", "cw");

  await pipe(readDir("scriptor/types"))
    .filter((m) => !m.path.endsWith("index.d.ts"))
    .map((m) => readFile(m.path))
    .map((m) => decoder.decode(m))
    .forEach((m) => file.write(m + "\n\n"));

  await file.flush();

  console.log({
    hello: "world",
  });
}
