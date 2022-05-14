declare module "pipe" {
  class Pipe<T> {
    constructor(iter: AsyncIterable<T>);

    map<R>(map: (item: T) => Promise<R> | R): this;
    filter(filter: (item: T) => Promise<boolean> | boolean): this;
    collect(): Promise<T[]>;
  }
}
