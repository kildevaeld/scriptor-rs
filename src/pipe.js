export class Pipe {
  static from(array) {
    return new Pipe(combine(array));
  }

  constructor(stream) {
    this.stream = stream;
  }

  filter(f) {
    this.stream = filter(this.stream, f);
    return this;
  }

  map(m) {
    this.stream = map(this.stream, m);
    return this;
  }

  flat() {
    this.stream = flatten(this.stream);
    return this;
  }

  async collect() {
    const out = [];
    for await (const item of this.stream) {
      out.push(item);
    }

    return out;
  }

  async fold(acc, init) {
    for await (const item of this.stream) {
      init = await acc(init, item);
    }
    return init;
  }

  async find(find) {
    for await (const item of this.stream) {
      if (await find(item)) {
        return item;
      }
    }
  }

  [Symbol.asyncIterator]() {
    return this.stream[Symbol.asyncIterator]();
  }
}

async function* asyncIter(iter) {
  const items = await Promise.all(iter);
  for (const item of items) {
    yield item;
  }
}

async function* map(stream, map) {
  for await (const next of stream) {
    yield await map(next);
  }
}

async function* filter(stream, filter) {
  for await (const next of stream) {
    if (await filter(next)) {
      yield next;
    }
  }
}

async function* array(a) {
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

export async function* combine(iterable) {
  iterable = await Promise.all(iterable);
  const asyncIterators = Array.from(iterable, (o) => {
    if (o[Symbol.asyncIterator]) {
      return o[Symbol.asyncIterator]();
    } else {
      return array(o);
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
  return new Pipe(results);
}
