declare module "fs" {
  export function readFile(path: string): Promise<Buffer>;

  export function writeFile(
    path: string,
    content: string | Uint8Array
  ): Promise<void>;

  class File {}

  export function open(path: string): Promise<File>;
}
