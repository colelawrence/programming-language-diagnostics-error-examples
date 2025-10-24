import { describe, test, expect, beforeAll, afterAll } from "vitest";
import { createRouter, createWebSocketAdaptor } from "./index";
import type { PathResult, ShortestPathParams } from "../../dist-types";
import { spawn, type ChildProcess } from "node:child_process";

describe("Router with WebSocket Adaptor", () => {
  let serverProcess: ChildProcess | null = null;

  beforeAll(async () => {
    // Start the WebSocket server
    return new Promise<void>((resolve, reject) => {
      serverProcess = spawn("cargo", ["run", "-p", "pathfinder-server"], {
        cwd: process.cwd(),
        stdio: ["ignore", "pipe", "pipe"],
      });

      let output = "";

      serverProcess.stdout?.on("data", (data) => {
        output += data.toString();
        if (output.includes("listening on")) {
          // Give it a moment to fully initialize
          setTimeout(() => resolve(), 500);
        }
      });

      serverProcess.stderr?.on("data", (data) => {
        console.error("Server stderr:", data.toString());
      });

      serverProcess.on("error", (error) => {
        reject(error);
      });

      // Timeout after 10 seconds
      setTimeout(() => {
        reject(new Error("Server failed to start within 10 seconds"));
      }, 10000);
    });
  }, 15000);

  afterAll(() => {
    if (serverProcess) {
      serverProcess.kill();
    }
  });

  test("can connect to WebSocket server", async () => {
    const router = createRouter({
      adaptor: createWebSocketAdaptor({
        url: "ws://127.0.0.1:10810",
      }),
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

    const results: PathResult[] = [];
    let errorMsg: string | undefined;

    await new Promise<void>((resolve) => {
      router.find_shortest_path(params).subscribe({
        next: (result) => {
          results.push(result);
        },
        error: (error) => {
          errorMsg = error;
          resolve();
        },
        complete: () => {
          resolve();
        },
      });

      // Timeout after 5 seconds
      setTimeout(() => resolve(), 5000);
    });

    expect(errorMsg).toBeUndefined();
    expect(results.length).toBeGreaterThan(0);
    expect(results[0].path).toEqual([0, 1]);

    router.dispose();
  }, 10000);

  test("find shortest path via WebSocket", async () => {
    const router = createRouter({
      adaptor: createWebSocketAdaptor({
        url: "ws://127.0.0.1:10810",
      }),
    });

    const params: ShortestPathParams = {
      points: [
        { x: 0.0, y: 0.0 },
        { x: 3.0, y: 0.0 },
        { x: 0.0, y: 4.0 },
      ],
      edges: [
        { from: 0, to: 1 },
        { from: 1, to: 2 },
        { from: 0, to: 2 },
      ],
      start_idx: 0,
      end_idx: 2,
    };

    const results: PathResult[] = [];

    await new Promise<void>((resolve) => {
      router.find_shortest_path(params).subscribe({
        next: (result) => results.push(result),
        complete: () => resolve(),
      });

      setTimeout(() => resolve(), 5000);
    });

    expect(results).toHaveLength(1);
    const result = results[0];
    expect(result.path).toEqual([0, 2]);
    expect(result.distance).toBeCloseTo(4.0, 5);

    router.dispose();
  }, 10000);

  test("handles no path found via WebSocket", async () => {
    const router = createRouter({
      adaptor: createWebSocketAdaptor({
        url: "ws://127.0.0.1:10810",
      }),
    });

    const params: ShortestPathParams = {
      points: [
        { x: 0.0, y: 0.0 },
        { x: 1.0, y: 0.0 },
        { x: 10.0, y: 10.0 },
      ],
      edges: [{ from: 0, to: 1 }],
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
        complete: () => resolve(),
      });

      setTimeout(() => resolve(), 5000);
    });

    expect(errorMsg).toBe("No path found");

    router.dispose();
  }, 10000);

  test("multiple concurrent requests", async () => {
    const router = createRouter({
      adaptor: createWebSocketAdaptor({
        url: "ws://127.0.0.1:10810",
      }),
    });

    const params1: ShortestPathParams = {
      points: [{ x: 0, y: 0 }, { x: 1, y: 0 }],
      edges: [{ from: 0, to: 1 }],
      start_idx: 0,
      end_idx: 1,
    };

    const params2: ShortestPathParams = {
      points: [{ x: 0, y: 0 }, { x: 2, y: 0 }],
      edges: [{ from: 0, to: 1 }],
      start_idx: 0,
      end_idx: 1,
    };

    const results1: PathResult[] = [];
    const results2: PathResult[] = [];

    await Promise.all([
      new Promise<void>((resolve) => {
        router.find_shortest_path(params1).subscribe({
          next: (result) => results1.push(result),
          complete: () => resolve(),
        });
      }),
      new Promise<void>((resolve) => {
        router.find_shortest_path(params2).subscribe({
          next: (result) => results2.push(result),
          complete: () => resolve(),
        });
      }),
    ]);

    expect(results1.length).toBeGreaterThan(0);
    expect(results2.length).toBeGreaterThan(0);

    router.dispose();
  }, 10000);
});
