declare module "pipe" {
  export type Result<T> = Promise<T> | T;
  export function pipe<T>(
    pipe: AsyncIterable<T> | Promise<AsyncIterable<T>>
  ): Pipe<T>;
  export class Pipe<T> {
    private _stream;
    static from(array: any): Pipe<any>;
    constructor(stream: AsyncIterable<T>);
    filter(filterFn: (item: T) => Result<boolean>): this;
    map<R>(mapFn: (item: T) => Result<R>): Pipe<R>;
    flat(): Pipe<FlatAsyncIterator<T, 1>>;
    collect(): Promise<T[]>;
    fold<R>(acc: (prev: R, cur: T) => Result<R>, init: R): Promise<R>;
    find(find: (item: T) => Result<boolean>): Promise<T | undefined>;
    join(joiner: string): Promise<string>;
    combine(stream: unknown[]): Pipe<T>;
    [Symbol.asyncIterator](): AsyncIterator<T, any, undefined>;
  }
  export function combine<T extends any[]>(
    iterable: T
  ): AsyncGenerator<any, void, unknown>;
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
}
