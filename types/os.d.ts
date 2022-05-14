declare module "os" {
  class Stdout implements Write {}
  class Stderr implements Write {}

  export const stdout: Stdout;
  export const stderr: Stderr;
}
