export type Result<T> = Promise<T> | T;

export function pipe<T>(
  pipe: AsyncIterable<T> | Promise<AsyncIterable<T>>
): Pipe<T> {
  return new Pipe(toAsync(pipe));
}

export class Pipe<T> {
  private _stream: AsyncIterable<T>;
  static from(array) {
    return new Pipe(combine(array));
  }

  constructor(stream: AsyncIterable<T>) {
    this._stream = stream;
  }

  filter(filterFn: (item: T) => Result<boolean>): this {
    this._stream = filter(this._stream, filterFn);
    return this;
  }

  map<R>(mapFn: (item: T) => Result<R>): Pipe<R> {
    this._stream = map(this._stream, mapFn) as unknown as AsyncIterable<T>;
    return this as unknown as Pipe<R>;
  }

  async forEach(fn: (item: T, idx: number) => void): Promise<void> {
    let enumerator = 0;
    for await (const next of this._stream) {
      fn(next, enumerator++);
    }
  }

  flat(): Pipe<FlatAsyncIterator<T, 1>> {
    this._stream = flatten(this._stream);
    return this as any;
  }

  async collect(): Promise<T[]> {
    const out = [];
    for await (const item of this._stream) {
      out.push(item);
    }

    return out;
  }

  async fold<R>(acc: (prev: R, cur: T) => Result<R>, init: R): Promise<R> {
    for await (const item of this._stream) {
      init = await acc(init, item);
    }
    return init;
  }

  async find(find: (item: T) => Result<boolean>) {
    for await (const item of this._stream) {
      if (await find(item)) {
        return item;
      }
    }
  }

  async join(joiner: string): Promise<string> {
    let out = "";
    for await (const item of this._stream) {
      out += joiner + String(item);
    }

    return out;
  }

  combine(stream: unknown[]): Pipe<T> {
    this._stream = combine([this._stream, ...stream]);
    return this as any;
  }

  [Symbol.asyncIterator]() {
    return this._stream[Symbol.asyncIterator]();
  }
}

async function* toAsync<T>(
  iter: AsyncIterable<T> | Promise<AsyncIterable<T>>
): AsyncIterable<T> {
  const items = await iter;
  for await (const item of items) {
    yield item;
  }
}

async function* asyncIter(iter) {
  const items = await Promise.all(iter);
  for (const item of items) {
    yield item;
  }
}

async function* map<T, R>(
  stream: AsyncIterable<T>,
  map: (item: T) => Result<R>
) {
  for await (const next of stream) {
    yield await map(next);
  }
}

async function* filter<T>(
  stream: AsyncIterable<T>,
  filter: (item: T) => Result<boolean>
) {
  for await (const next of stream) {
    if (await filter(next)) {
      yield next;
    }
  }
}

export async function* iter<T>(a: Iterable<T>): AsyncIterable<T> {
  for (const item of a) {
    yield item;
  }
}

async function* flatten(stream) {
  for await (const item of stream) {
    if (typeof item[Symbol.asyncIterator] == "function") {
      for await (const inner of item) {
        yield inner;
      }
    } else {
      yield item;
    }
  }
}

export async function* combine<T extends any[]>(iterable: T) {
  iterable = await Promise.all(iterable);
  const asyncIterators = Array.from(iterable, (o) => {
    if (o[Symbol.asyncIterator]) {
      return o[Symbol.asyncIterator]();
    } else {
      return iter(o);
    }
  });
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
}

type FlatAsyncIterator<Arr, Depth extends number> = {
  done: Arr;
  recur: Arr extends AsyncIterable<infer InnerArr>
    ? FlatAsyncIterator<
        InnerArr,
        [
          -1,
          0,
          1,
          2,
          3,
          4,
          5,
          6,
          7,
          8,
          9,
          10,
          11,
          12,
          13,
          14,
          15,
          16,
          17,
          18,
          19,
          20
        ][Depth]
      >
    : Arr;
}[Depth extends -1 ? "done" : "recur"];
