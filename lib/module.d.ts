declare module "os" {
  interface Write {
    write(data: Uint8Array): Promise<void>;
  }

  export const stdout: Write;
}

declare function delay(n: number): Promise<void>;
