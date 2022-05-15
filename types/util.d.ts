declare module "util" {
  export function format(a: unknown): string;

  export interface ConsoleApi {
    log(...args: unknown[]);
  }

  export function createConsole(print: (arg: string) => void): ConsoleApi;
}
