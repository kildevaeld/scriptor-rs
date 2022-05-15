export function format(input: unknown, quote = false): string {
  if (input === null) return "null";
  if (input === undefined) return "undefined";
  if (typeof input == "string") return quote ? "'" + input + "'" : input;

  if (typeof input == "number") return input + "";

  if (Array.isArray(input)) {
    return `[ ${input.map((m) => format(m, true)).join(", ")} ]`;
  }

  if (typeof input === "function") {
    return `[Function ${input.name}]`;
  }

  if ("toString" in (input as any)) {
    return String(input);
  } else {
    return "object";
  }
}

export interface Console {
  log(...args: unknown[]): void;
}

export function createConsole(print: (input: string) => void): Console {
  return {
    log(...args: unknown[]): void {
      const out = args
        .map((m) => {
          return format(m);
        })
        .join(" ");

      print(out + "\n");
      //   print(args.map((m) => format(m)).join(" "));
    },
  };
}
