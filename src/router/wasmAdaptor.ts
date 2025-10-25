import type { Adaptor, RequestEnum, WireResponse } from "./types";
import * as wasm from "../../pkg/wasm_pathfinder";
import init from "../../pkg/wasm_pathfinder";

export interface WasmAdaptorOptions {
  /** Optional custom WASM initialization */
  wasmInit?: typeof init;
}

export function createWasmAdaptor(
  options: WasmAdaptorOptions = {},
): Adaptor {
  let initialized = false;

  return {
    async init() {
      if (initialized) return;

      const initFn = options.wasmInit || init;
      await initFn();

      initialized = true;
    },

    sendRequest(request: RequestEnum) {
      if (!initialized) {
        throw new Error("WASM adaptor not initialized. Call init() first.");
      }

      return {
        subscribe: (observer) => {
          // Send the request to WASM with a callback
          wasm.send_request(request, (response: WireResponse) => {
            const [id, responseEnum] = response;
            
            if ("N" in responseEnum) {
              observer.next?.(response);
            } else if ("Error" in responseEnum) {
              observer.error?.(responseEnum.Error);
            } else if ("Complete" in responseEnum) {
              observer.complete?.(responseEnum.Complete);
            } else if ("Aborted" in responseEnum) {
              observer.error?.(responseEnum.Aborted);
            }
          });
        },
      };
    },

    dispose() {
      // No cleanup needed
    },
  };
}

/** Default WASM adaptor instance for convenience */
export const wasmAdaptor = createWasmAdaptor();
