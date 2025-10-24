import type { Observable, EnhancedObservable } from "./types";

export function enhanceObservable<T>(
  observable: Observable<T>,
): EnhancedObservable<T> {
  return {
    subscribe: observable.subscribe.bind(observable),

    first(options?: { signal?: AbortSignal }): Promise<T> {
      return new Promise((resolve, reject) => {
        let resolved = false;
        const cleanup = () => {
          resolved = true;
          options?.signal?.removeEventListener("abort", onAbort);
        };

        const onAbort = () => {
          cleanup();
          reject(new Error("Aborted"));
        };

        options?.signal?.addEventListener("abort", onAbort);

        observable.subscribe({
          next: (value) => {
            if (!resolved) {
              cleanup();
              resolve(value);
            }
          },
          error: (error) => {
            cleanup();
            reject(new Error(error));
          },
          complete: () => {
            if (!resolved) {
              cleanup();
              reject(new Error("Observable completed without emitting a value"));
            }
          },
        });
      });
    },

    all(options?: { signal?: AbortSignal }): Promise<T[]> {
      return new Promise((resolve, reject) => {
        const values: T[] = [];
        let completed = false;

        const cleanup = () => {
          completed = true;
          options?.signal?.removeEventListener("abort", onAbort);
        };

        const onAbort = () => {
          cleanup();
          reject(new Error("Aborted"));
        };

        options?.signal?.addEventListener("abort", onAbort);

        observable.subscribe({
          next: (value) => {
            if (!completed) {
              values.push(value);
            }
          },
          error: (error) => {
            cleanup();
            reject(new Error(error));
          },
          complete: () => {
            cleanup();
            resolve(values);
          },
        });
      });
    },

    each(
      fn: (value: T) => void,
      options?: { signal?: AbortSignal },
    ): AbortController {
      const controller = new AbortController();
      let stopped = false;

      const cleanup = () => {
        stopped = true;
        options?.signal?.removeEventListener("abort", onAbort);
      };

      const onAbort = () => {
        cleanup();
        controller.abort();
      };

      options?.signal?.addEventListener("abort", onAbort);

      controller.signal.addEventListener("abort", cleanup);

      observable.subscribe({
        next: (value) => {
          if (!stopped) {
            fn(value);
          }
        },
        error: () => {
          cleanup();
        },
        complete: () => {
          cleanup();
        },
      });

      return controller;
    },
  };
}
