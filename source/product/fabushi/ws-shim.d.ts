declare module "ws" {
  export default class WebSocket {
    constructor(address: string, options?: { readonly headers?: Readonly<Record<string, string>> });
    readonly readyState: number;
    send(data: string, callback?: (error?: Error) => void): void;
    close(code?: number, reason?: string): void;
    terminate(): void;
    on(event: string, listener: (...args: unknown[]) => void): this;
  }
}
