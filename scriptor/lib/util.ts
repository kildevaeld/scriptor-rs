export function isObjectLike(val: any): val is object {
  return val === Object(val);
}

export function isObject(val: any): val is object {
  return val != null && typeof val === "object" && Array.isArray(val) === false;
}

function isObjectObject(o: any) {
  return (
    isObject(o) === true &&
    Object.prototype.toString.call(o) === "[object Object]"
  );
}

export function isPlainObject(o: any): o is object {
  var ctor: any, prot: any;

  if (isObjectObject(o) === false) return false;

  // If has modified constructor
  ctor = o.constructor;
  if (typeof ctor !== "function") return false;

  // If has modified prototype
  prot = ctor.prototype;
  if (isObjectObject(prot) === false) return false;

  // If constructor does not have an Object-specific method
  if (prot.hasOwnProperty("isPrototypeOf") === false) {
    return false;
  }

  // Most likely a plain Object
  return true;
}

export function format(input: unknown, quote = false): string {
  if (input === null) return "null";
  if (input === undefined) return "undefined";
  if (typeof input == "string") return quote ? "'" + input + "'" : input;

  if (typeof input == "number") return input + "";

  if (typeof input == "boolean") return input + "";

  if (Array.isArray(input)) {
    return `[ ${input.map((m) => format(m, true)).join(", ")} ]`;
  }

  if (typeof input === "function") {
    return `[Function ${input.name}]`;
  }
  if (isPlainObject(input)) {
    const buf = [];
    for (const key in input) {
      buf.push(`${key}: ${format(input[key], true)}`);
    }
    return `{ ${buf.join(", ")} }`;
  }

  if ("toString" in (input as any)) {
    return String(input);
  } else {
    return "object";
  }
}

export interface Console {
  log(...args: unknown[]): void;
  warn(...args: unknown[]): void;
}

function factory(echo: (input: string) => void) {
  return (...args: unknown[]) => {
    const out = args
      .map((m) => {
        return format(m);
      })
      .join(" ");
    echo(out + "\n");
  };
}

export function createConsole(
  stdout: (input: string) => void,
  stderr?: (input: string) => void
): Console {
  const stdoutFn = factory(stdout);
  const stderrFn = stderr ? factory(stderr) : stdoutFn;

  return {
    log: stdoutFn,
    warn: stderrFn,
  };
}
