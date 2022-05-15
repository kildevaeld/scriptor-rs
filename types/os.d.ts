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
