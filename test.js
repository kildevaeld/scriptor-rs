import { open, readDir } from "fs";
import { stdout } from "os";
import { Pipe, combine } from "pipe";

const decoder = new TextDecoder();

export async function main() {
  const lines = [
    open("test.js").then((m) => m.lines()),
    open("Cargo.toml").then((m) => m.lines()),
  ];

  const pipe = Pipe.from(lines);

  for await (let line of pipe) {
    print(line);
  }

  //   const listDir = await readDir(".");

  //   for await (const entry of listDir) {
  //     // print("line " + entry.path());
  //   }
}
