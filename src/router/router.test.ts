import { describe, test, expect, beforeAll } from "vitest";
import { createRouter, createWasmAdaptor } from "./index";
import type { PathResult, Point, Edge, ShortestPathParams } from "../../dist-types";
import init from "../../pkg/wasm_pathfinder";
import { readFile } from "node:fs/promises";
import { resolve } from "node:path";

describe("Router with WASM Adaptor", () => {
  let wasmInit: typeof init;

  beforeAll(async () => {
    const wasmPath = resolve(__dirname, "../../pkg/wasm_pathfinder_bg.wasm");
    const wasmBuffer = await readFile(wasmPath);
    
    // Create a reusable init function that uses the buffer
    wasmInit = () => init(wasmBuffer);
    await wasmInit();
  });

  test("createRouter returns Router interface", () => {
    const router = createRouter({
      adaptor: createWasmAdaptor({ wasmInit }),
    });

    expect(router).toBeDefined();
    expect(router.find_shortest_path).toBeDefined();
    expect(router.dispose).toBeDefined();
  });

  test("find shortest path - simple triangle", async () => {
    const router = createRouter({
      adaptor: createWasmAdaptor({ wasmInit }),
    });

    const points: Point[] = [
      { x: 0.0, y: 0.0 },
      { x: 3.0, y: 0.0 },
      { x: 0.0, y: 4.0 },
    ];

    const edges: Edge[] = [
      { from: 0, to: 1 }, // distance 3
      { from: 1, to: 2 }, // distance 5
      { from: 0, to: 2 }, // distance 4 (direct)
    ];

    const params: ShortestPathParams = {
      points,
      edges,
      start_idx: 0,
      end_idx: 2,
    };

    const results: PathResult[] = [];
    let errorMsg: string | undefined;
    let completeMsg: string | undefined;

    await new Promise<void>((resolve) => {
      router.find_shortest_path(params).subscribe({
        next: (result) => results.push(result),
        error: (error) => {
          errorMsg = error;
          resolve();
        },
        complete: (notes) => {
          completeMsg = notes;
          resolve();
        },
      });
    });

    expect(errorMsg).toBeUndefined();
    expect(completeMsg).toBe("Path found successfully");
    expect(results).toHaveLength(1);

    const result = results[0];
    expect(result.path).toEqual([0, 2]); // Direct path is shortest
    expect(result.distance).toBeCloseTo(4.0, 5);

    router.dispose();
  });

  test("find shortest path - no path exists", async () => {
    const router = createRouter({
      adaptor: createWasmAdaptor({ wasmInit }),
    });

    const points: Point[] = [
      { x: 0.0, y: 0.0 },
      { x: 1.0, y: 0.0 },
      { x: 10.0, y: 10.0 }, // Disconnected
    ];

    const edges: Edge[] = [
      { from: 0, to: 1 },
      // No edge to point 2
    ];

    const params: ShortestPathParams = {
      points,
      edges,
      start_idx: 0,
      end_idx: 2,
    };

    let errorMsg: string | undefined;

    await new Promise<void>((resolve) => {
      router.find_shortest_path(params).subscribe({
        error: (error) => {
          errorMsg = error;
          resolve();
        },
        complete: () => {
          resolve();
        },
      });
    });

    expect(errorMsg).toBe("No path found");

    router.dispose();
  });

  test("find shortest path - longer path", async () => {
    const router = createRouter({
      adaptor: createWasmAdaptor({ wasmInit }),
    });

    const points: Point[] = [
      { x: 0.0, y: 0.0 },
      { x: 1.0, y: 0.0 },
      { x: 2.0, y: 0.0 },
      { x: 3.0, y: 0.0 },
      { x: 4.0, y: 0.0 },
      { x: 5.0, y: 0.0 },
    ];

    const edges: Edge[] = [
      { from: 0, to: 1 },
      { from: 1, to: 2 },
      { from: 2, to: 3 },
      { from: 3, to: 4 },
      { from: 4, to: 5 },
    ];

    const params: ShortestPathParams = {
      points,
      edges,
      start_idx: 0,
      end_idx: 5,
    };

    const results: PathResult[] = [];

    await new Promise<void>((resolve) => {
      router.find_shortest_path(params).subscribe({
        next: (result) => results.push(result),
        complete: () => resolve(),
      });
    });

    expect(results).toHaveLength(1);
    const result = results[0];
    expect(result.path).toEqual([0, 1, 2, 3, 4, 5]);
    expect(result.distance).toBeCloseTo(5.0, 5);

    router.dispose();
  });

  test("router can handle multiple calls", async () => {
    const router = createRouter({
      adaptor: createWasmAdaptor({ wasmInit }),
    });

    const params: ShortestPathParams = {
      points: [
        { x: 0.0, y: 0.0 },
        { x: 1.0, y: 0.0 },
      ],
      edges: [{ from: 0, to: 1 }],
      start_idx: 0,
      end_idx: 1,
    };

    // First call
    const results1: PathResult[] = [];
    await new Promise<void>((resolve) => {
      router.find_shortest_path(params).subscribe({
        next: (result) => {
          results1.push(result);
          resolve(); // Resolve after receiving result
        },
        error: () => resolve(),
      });
    });

    // Second call
    const results2: PathResult[] = [];
    await new Promise<void>((resolve) => {
      router.find_shortest_path(params).subscribe({
        next: (result) => {
          results2.push(result);
          resolve(); // Resolve after receiving result
        },
        error: () => resolve(),
      });
    });

    expect(results1).toHaveLength(1);
    expect(results2).toHaveLength(1);
    expect(results1[0].path).toEqual([0, 1]);
    expect(results2[0].path).toEqual([0, 1]);

    router.dispose();
  });

  test("dispose cleans up resources", () => {
    const router = createRouter({
      adaptor: createWasmAdaptor({ wasmInit }),
    });

    expect(() => router.dispose()).not.toThrow();
  });

  test("observer callbacks are optional", async () => {
    const router = createRouter({
      adaptor: createWasmAdaptor({ wasmInit }),
    });

    const params: ShortestPathParams = {
      points: [
        { x: 0.0, y: 0.0 },
        { x: 1.0, y: 0.0 },
      ],
      edges: [{ from: 0, to: 1 }],
      start_idx: 0,
      end_idx: 1,
    };

    // Should not throw even with no callbacks
    await new Promise<void>((resolve) => {
      setTimeout(resolve, 100);
      router.find_shortest_path(params).subscribe({});
    });

    router.dispose();
  });

  test("grid pathfinding with multiple paths", async () => {
    const router = createRouter({
      adaptor: createWasmAdaptor({ wasmInit }),
    });

    // Create a 3x3 grid
    // 0 - 1 - 2
    // |   |   |
    // 3 - 4 - 5
    // |   |   |
    // 6 - 7 - 8
    const points: Point[] = [
      { x: 0, y: 0 }, { x: 1, y: 0 }, { x: 2, y: 0 },
      { x: 0, y: 1 }, { x: 1, y: 1 }, { x: 2, y: 1 },
      { x: 0, y: 2 }, { x: 1, y: 2 }, { x: 2, y: 2 },
    ];

    const edges: Edge[] = [
      // Horizontal edges
      { from: 0, to: 1 }, { from: 1, to: 2 },
      { from: 3, to: 4 }, { from: 4, to: 5 },
      { from: 6, to: 7 }, { from: 7, to: 8 },
      // Vertical edges
      { from: 0, to: 3 }, { from: 3, to: 6 },
      { from: 1, to: 4 }, { from: 4, to: 7 },
      { from: 2, to: 5 }, { from: 5, to: 8 },
    ];

    const params: ShortestPathParams = {
      points,
      edges,
      start_idx: 0,
      end_idx: 8,
    };

    const results: PathResult[] = [];
    let errorOccurred = false;

    await new Promise<void>((resolve) => {
      router.find_shortest_path(params).subscribe({
        next: (result) => {
          results.push(result);
          resolve();
        },
        error: (err) => {
          console.error("Error in grid test:", err);
          errorOccurred = true;
          resolve();
        },
        complete: () => {
          if (!errorOccurred && results.length === 0) {
            console.log("Complete called but no results");
          }
          resolve();
        },
      });

      // Safety timeout
      setTimeout(() => {
        console.log("Timeout reached, results:", results.length);
        resolve();
      }, 4000);
    });

    expect(errorOccurred).toBe(false);
    expect(results.length).toBeGreaterThan(0);
    
    if (results.length > 0) {
      const result = results[0];
      expect(result.path[0]).toBe(0);
      expect(result.path[result.path.length - 1]).toBe(8);
      expect(result.distance).toBeCloseTo(4.0, 5);
    }

    router.dispose();
  });
});
