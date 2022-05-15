declare function sleep(ms: number): Promise<void>;

declare class TextEncoder {
  encode(text: string): Uint8Array;
}

declare class TextDecoder {
  decode(text: Uint8Array): string;
}
