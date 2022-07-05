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
