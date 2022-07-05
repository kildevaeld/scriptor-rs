import { readFile, readDir, open } from "fs";
import { pipe } from "pipe";

const decoder = new TextDecoder();

export async function main(greet) {
  const file = await open("scriptor.d.ts", "cw");

  console.log("Started", greet);

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
