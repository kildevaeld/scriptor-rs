

declare function sleep(ms: number): Promise<void>;

declare class TextEncoder {
  encode(text: string): Uint8Array;
}

declare class TextDecoder {
  decode(text: Uint8Array): string;
}


declare module "util" {
  export function format(a: unknown): string;

  export interface ConsoleApi {
    log(...args: unknown[]);
  }

  export function createConsole(print: (arg: string) => void): ConsoleApi;
}


declare module "os" {
  class Stdout implements Write {
    flush(): Promise<void>;
    write(data: Uint8Array): Promise<void>;
  }
  class Stderr implements Write {
    flush(): Promise<void>;
    write(data: Uint8Array): Promise<void>;
  }

  export const stdout: Stdout;
  export const stderr: Stderr;
}


declare module "fs" {
  export function readFile(path: string): Promise<Uint8Array>;

  export function writeFile(
    path: string,
    content: string | Uint8Array
  ): Promise<void>;

  class File {
    read(): Promise<Uint8Array>;
    lines(): AsyncIterable<string>;
  }

  class DirEntry {
    readonly path: string;
  }

  class ReadDir {
    [Symbol.asyncIterator](): AsyncIterator<DirEntry>;
  }

  export function open(path: string): Promise<File>;

  export function readDir(path: string): Promise<ReadDir>;
}


declare interface Read {
  read(): Promise<Uint8Array>;
}

declare interface Write {
  write(data: Uint8Array): Promise<void>;
  flush(): Promise<void>;
}


declare module "http" {
  export class Url {
    constructor(url: string);
  }

  export type Method =
    | "GET"
    | "POST"
    | "PUT"
    | "PATCH"
    | "DELETE"
    | "HEAD"
    | "OPTIONS";

  export class Request {
    constructor(url: Url);
    constructor(url: string);

    headers: Headers;
    method: Method;
  }
  export class Response {
    readonly status: number;
    readonly headers: Headers;
    text(): Promise<string>;
  }

  export class Headers {
    set(name: string, value: string): void;
    get(name: string): string | undefined;
  }

  export class Client {
    send(req: Request): Promise<Response>;
  }
}


/**
Matches a JSON object.
This type can be useful to enforce some input to be JSON-compatible or as a super-type to be extended from. Don't use this as a direct return type as the user would have to double-cast it: `jsonObject as unknown as CustomResponse`. Instead, you could extend your CustomResponse type from it to ensure your type only uses JSON-compatible types: `interface CustomResponse extends JsonObject { … }`.
@category JSON
*/
declare type JsonObject = { [Key in string]?: JsonValue };

/**
Matches a JSON array.
@category JSON
*/
declare type JsonArray = JsonValue[];

/**
Matches any valid JSON primitive value.
@category JSON
*/
declare type JsonPrimitive = string | number | boolean | null;

/**
Matches any valid JSON value.
@see `Jsonify` if you need to transform a type to one that is assignable to `JsonValue`.
@category JSON
*/
declare type JsonValue = JsonPrimitive | JsonObject | JsonArray;


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
