import type { Adaptor, Router, Observable } from "./types";
import { enhanceObservable } from "./enhancedObservable";

export interface CreateRouterOptions {
  adaptor: Adaptor;
}

export function createRouter(options: CreateRouterOptions): Router {
  const { adaptor } = options;
  let requestId = 0;
  let initialized = false;

  // Lazy initialization wrapper
  async function ensureInitialized() {
    if (!initialized) {
      await adaptor.init();
      initialized = true;
    }
  }

  // Create a proxy that handles all router method calls dynamically
  return new Proxy({
    dispose() {
      adaptor.dispose?.();
    },
  } as Router, {
    get(target, methodName: string) {
      // Return the actual dispose method
      if (methodName === 'dispose') {
        return target.dispose;
      }

      // For any other method, create a dynamic router call
      return (params: any) => {
        const obs: Observable<any> = {
          subscribe: (observer) => {
            ensureInitialized()
              .then(() => {
                requestId++;
                const request: import("./types").RequestEnum = {
                  Call: [requestId, { [methodName]: params } as any],
                };

                adaptor.sendRequest(request).subscribe({
                  next: (response) => {
                    const [, responseEnum] = response;
                    if ("N" in responseEnum && methodName in responseEnum.N) {
                      observer.next?.((responseEnum.N as any)[methodName]);
                    }
                  },
                  ...(observer.error && { error: observer.error }),
                  ...(observer.complete && { complete: observer.complete }),
                });
              })
              .catch((error) => {
                observer.error?.(
                  error instanceof Error ? error.message : String(error),
                );
              });
          },
        };
        return enhanceObservable(obs);
      };
    },
  });
}
