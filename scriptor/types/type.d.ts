declare interface Read {
  read(): Promise<Uint8Array>;
}

declare interface Write {
  write(data: Uint8Array): Promise<void>;
  flush(): Promise<void>;
}
