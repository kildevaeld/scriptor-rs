import { open } from "fs";
import { stdout } from "os";

const decoder = new TextDecoder();

export async function main() {
  const lines = await Promise.all([
    open("test.js").then((m) => m.lines()),
    open("Cargo.toml").then((m) => m.lines()),
  ]);

  for await (let line of combine(lines)) {
    print(line);
  }

  await delay(1000);
}

async function* combine(iterable) {
  const asyncIterators = Array.from(iterable, (o) => o[Symbol.asyncIterator]());
  const results = [];
  let count = asyncIterators.length;
  const never = new Promise(() => {});
  function getNext(asyncIterator, index) {
    return asyncIterator.next().then((result) => ({
      index,
      result,
    }));
  }
  const nextPromises = asyncIterators.map(getNext);
  try {
    while (count) {
      const { index, result } = await Promise.race(nextPromises);
      if (result.done) {
        nextPromises[index] = never;
        results[index] = result.value;
        count--;
      } else {
        nextPromises[index] = getNext(asyncIterators[index], index);
        yield result.value;
      }
    }
  } finally {
    for (const [index, iterator] of asyncIterators.entries())
      if (nextPromises[index] != never && iterator.return != null)
        iterator.return();
    // no await here - see https://github.com/tc39/proposal-async-iteration/issues/126
  }
  return results;
}
