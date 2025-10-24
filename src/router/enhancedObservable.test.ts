import { describe, it, expect, vi } from "vitest";
import { enhanceObservable } from "./enhancedObservable";
import type { Observable } from "./types";

describe("enhanceObservable", () => {
  describe("first()", () => {
    it("resolves with the first emission", async () => {
      const obs: Observable<number> = {
        subscribe: (observer) => {
          observer.next?.(1);
          observer.next?.(2);
          observer.complete?.("done");
        },
      };

      const enhanced = enhanceObservable(obs);
      const result = await enhanced.first();
      expect(result).toBe(1);
    });

    it("rejects if completed without emission", async () => {
      const obs: Observable<number> = {
        subscribe: (observer) => {
          observer.complete?.("done");
        },
      };

      const enhanced = enhanceObservable(obs);
      await expect(enhanced.first()).rejects.toThrow(
        "Observable completed without emitting a value",
      );
    });

    it("respects AbortSignal and rejects with 'Aborted'", async () => {
      const obs: Observable<number> = {
        subscribe: () => {},
      };

      const controller = new AbortController();
      const enhanced = enhanceObservable(obs);
      const promise = enhanced.first({ signal: controller.signal });

      controller.abort();

      await expect(promise).rejects.toThrow("Aborted");
    });

    it("propagates errors", async () => {
      const obs: Observable<number> = {
        subscribe: (observer) => {
          observer.error?.("Test error");
        },
      };

      const enhanced = enhanceObservable(obs);
      await expect(enhanced.first()).rejects.toThrow("Test error");
    });
  });

  describe("all()", () => {
    it("collects all emissions until complete", async () => {
      const obs: Observable<number> = {
        subscribe: (observer) => {
          observer.next?.(1);
          observer.next?.(2);
          observer.next?.(3);
          observer.complete?.("done");
        },
      };

      const enhanced = enhanceObservable(obs);
      const result = await enhanced.all();
      expect(result).toEqual([1, 2, 3]);
    });

    it("returns empty array if no emissions", async () => {
      const obs: Observable<number> = {
        subscribe: (observer) => {
          observer.complete?.("done");
        },
      };

      const enhanced = enhanceObservable(obs);
      const result = await enhanced.all();
      expect(result).toEqual([]);
    });

    it("respects AbortSignal", async () => {
      const obs: Observable<number> = {
        subscribe: () => {},
      };

      const controller = new AbortController();
      const enhanced = enhanceObservable(obs);
      const promise = enhanced.all({ signal: controller.signal });

      controller.abort();

      await expect(promise).rejects.toThrow("Aborted");
    });

    it("propagates errors", async () => {
      const obs: Observable<number> = {
        subscribe: (observer) => {
          observer.next?.(1);
          observer.error?.("Test error");
        },
      };

      const enhanced = enhanceObservable(obs);
      await expect(enhanced.all()).rejects.toThrow("Test error");
    });
  });

  describe("each()", () => {
    it("invokes callback for each emission", () => {
      const obs: Observable<number> = {
        subscribe: (observer) => {
          observer.next?.(1);
          observer.next?.(2);
          observer.next?.(3);
          observer.complete?.("done");
        },
      };

      const enhanced = enhanceObservable(obs);
      const callback = vi.fn();
      enhanced.each(callback);

      expect(callback).toHaveBeenCalledTimes(3);
      expect(callback).toHaveBeenNthCalledWith(1, 1);
      expect(callback).toHaveBeenNthCalledWith(2, 2);
      expect(callback).toHaveBeenNthCalledWith(3, 3);
    });

    it("returned AbortController stops further callbacks", () => {
      let emitNext: (value: number) => void = () => {};

      const obs: Observable<number> = {
        subscribe: (observer) => {
          emitNext = (value: number) => observer.next?.(value);
        },
      };

      const enhanced = enhanceObservable(obs);
      const callback = vi.fn();
      const controller = enhanced.each(callback);

      emitNext(1);
      expect(callback).toHaveBeenCalledTimes(1);

      controller.abort();

      emitNext(2);
      expect(callback).toHaveBeenCalledTimes(1);
    });

    it("respects external AbortSignal", () => {
      let emitNext: (value: number) => void = () => {};

      const obs: Observable<number> = {
        subscribe: (observer) => {
          emitNext = (value: number) => observer.next?.(value);
        },
      };

      const externalController = new AbortController();
      const enhanced = enhanceObservable(obs);
      const callback = vi.fn();
      enhanced.each(callback, { signal: externalController.signal });

      emitNext(1);
      expect(callback).toHaveBeenCalledTimes(1);

      externalController.abort();

      emitNext(2);
      expect(callback).toHaveBeenCalledTimes(1);
    });

    it("stops on error", () => {
      let emitNext: (value: number) => void = () => {};
      let emitError: () => void = () => {};

      const obs: Observable<number> = {
        subscribe: (observer) => {
          emitNext = (value: number) => observer.next?.(value);
          emitError = () => observer.error?.("Test error");
        },
      };

      const enhanced = enhanceObservable(obs);
      const callback = vi.fn();
      enhanced.each(callback);

      emitNext(1);
      expect(callback).toHaveBeenCalledTimes(1);

      emitError();

      emitNext(2);
      expect(callback).toHaveBeenCalledTimes(1);
    });
  });

  describe("subscribe() still works", () => {
    it("can still use traditional subscribe pattern", () => {
      const obs: Observable<number> = {
        subscribe: (observer) => {
          observer.next?.(1);
          observer.next?.(2);
          observer.complete?.("done");
        },
      };

      const enhanced = enhanceObservable(obs);
      const callback = vi.fn();
      enhanced.subscribe({ next: callback });

      expect(callback).toHaveBeenCalledTimes(2);
      expect(callback).toHaveBeenNthCalledWith(1, 1);
      expect(callback).toHaveBeenNthCalledWith(2, 2);
    });
  });
});
