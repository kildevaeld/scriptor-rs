import { readFile, readDir, writeFile } from "fs";
import { stdout } from "os";
import { pipe } from "pipe";
import { createConsole } from "util";

const decoder = new TextDecoder();
const encoder = new TextEncoder();

const console = createConsole((arg) => {
  stdout.write(encoder.encode(arg));
});

export async function main() {
  console.log("Started");
  const lines = await pipe(readDir("types"))
    .filter((m) => !m.path.endsWith("index.d.ts"))
    .map((m) => readFile(m.path))
    .map((m) => decoder.decode(m))
    .join("\n\n");

  console.log(lines);

  await writeFile("scriptor.d.ts", lines);
}
