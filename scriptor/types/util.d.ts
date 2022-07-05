declare module "util" {
  export function isPlainObject(a: unknown): a is Record<string, unknown>;
  export function isObject(a: unknown): a is object;

  export function format(a: unknown): string;

  export interface ConsoleApi {
    log(...args: unknown[]);
  }

  export function createConsole(
    stdout: (arg: string) => void,
    stderr?: (arg: string) => void
  ): ConsoleApi;
}
