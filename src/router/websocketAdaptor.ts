import type { Adaptor, RequestEnum, WireResponse } from "./types";

export interface WebSocketAdaptorOptions {
  url: string;
  /** Optional WebSocket protocols */
  protocols?: string | string[];
  /** Auto-reconnect on disconnect (default: false) */
  autoReconnect?: boolean;
  /** Reconnect delay in ms (default: 1000) */
  reconnectDelay?: number;
}

export function createWebSocketAdaptor(
  options: WebSocketAdaptorOptions,
): Adaptor {
  let ws: WebSocket | null = null;
  const observers = new Map<
    number,
    {
      next: (value: WireResponse) => void;
    }
  >();

  function connect(): Promise<void> {
    return new Promise((resolve, reject) => {
      try {
        ws = new WebSocket(options.url, options.protocols);

        ws.onopen = () => {
          console.log("WebSocket connected");
          resolve();
        };

        ws.onmessage = (event) => {
          try {
            const response = JSON.parse(event.data) as WireResponse;
            const [id] = response;
            const observer = observers.get(id);

            if (observer) {
              observer.next?.(response);
            }
          } catch (e) {
            console.error("Failed to parse WebSocket message:", e);
          }
        };

        ws.onerror = (error) => {
          console.error("WebSocket error:", error);
          reject(new Error("WebSocket connection failed"));
        };

        ws.onclose = () => {
          console.log("WebSocket disconnected");
          if (options.autoReconnect) {
            setTimeout(() => {
              connect().catch(console.error);
            }, options.reconnectDelay || 1000);
          }
        };
      } catch (e) {
        reject(e);
      }
    });
  }

  return {
    async init() {
      await connect();
    },

    sendRequest(request: RequestEnum) {
      return {
        subscribe: (observer) => {
          if (!ws || ws.readyState !== WebSocket.OPEN) {
            observer.error?.("WebSocket not connected");
            return;
          }

          // Extract request ID from the request
          const requestId =
            "Call" in request ? request.Call[0] : request.Abort[0];

          // Store internal observer - just forward the WireResponse
          const internalObserver = {
            next: (response: WireResponse) => {
              observer.next?.(response);
              
              // Clean up observer on terminal responses
              const [id, responseEnum] = response;
              if ("Error" in responseEnum || "Complete" in responseEnum || "Aborted" in responseEnum) {
                observers.delete(id);
              }
            },
          };
          observers.set(requestId, internalObserver);

          // Send request over WebSocket
          const json = JSON.stringify(request);
          ws.send(json);
        },
      };
    },

    dispose() {
      observers.clear();
      if (ws) {
        ws.close();
        ws = null;
      }
    },
  };
}
