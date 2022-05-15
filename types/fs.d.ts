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
