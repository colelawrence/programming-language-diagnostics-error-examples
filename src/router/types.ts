import type {
  CallGen,
  ResponseNextGen,
  Observable,
  EnhancedObservable,
  Router,
} from "../../dist-types/router.gen";

/** Wire protocol types matching Rust ResponseEnum/RequestEnum */
export type WireResponse = [
  number, // request_id
  ResponseEnum,
];

export type ResponseEnum =
  | { Aborted: string }
  | { Error: string }
  | { Complete: string }
  | { N: ResponseNextGen };

export type RequestEnum =
  | { Abort: [number, string] }
  | { Call: [number, CallGen] };

/** Transport adaptor interface - implementations handle sending/receiving */
export interface Adaptor {
  /** Initialize the adaptor (e.g., connect to WebSocket, init WASM) */
  init(): Promise<void>;

  /** Send a request and get back an observable for responses */
  sendRequest(request: RequestEnum): Observable<WireResponse>;

  /** Optional cleanup */
  dispose?(): void;
}

// Re-export generated types for convenience
export type { CallGen, ResponseNextGen, Observable, EnhancedObservable, Router };
