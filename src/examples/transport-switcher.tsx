import { useState, useEffect } from "react";
import {
  createRouter,
  createWasmAdaptor,
  createWebSocketAdaptor,
  type Router,
} from "../router";
import type { PathResult } from "../../dist-types";

type Transport = "wasm" | "websocket";

export function TransportSwitcher() {
  const [transport, setTransport] = useState<Transport>("wasm");
  const [router, setRouter] = useState<Router | null>(null);
  const [result, setResult] = useState<PathResult | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    // Create router based on selected transport
    const newRouter = createRouter({
      adaptor:
        transport === "wasm"
          ? createWasmAdaptor()
          : createWebSocketAdaptor({
              url: "ws://localhost:10810",
              autoReconnect: true,
            }),
    });

    setRouter(newRouter);

    return () => {
      newRouter.dispose();
    };
  }, [transport]);

  const runPathfinding = () => {
    if (!router) return;

    setLoading(true);
    setError(null);
    setResult(null);

    const startTime = performance.now();

    router
      .find_shortest_path({
        points: [
          { x: 0, y: 0 },
          { x: 1, y: 0 },
          { x: 2, y: 0 },
          { x: 0, y: 1 },
          { x: 1, y: 1 },
          { x: 2, y: 1 },
        ],
        edges: [
          { from: 0, to: 1 },
          { from: 1, to: 2 },
          { from: 0, to: 3 },
          { from: 1, to: 4 },
          { from: 2, to: 5 },
          { from: 3, to: 4 },
          { from: 4, to: 5 },
        ],
        start_idx: 0,
        end_idx: 5,
      })
      .first()
      .then((pathResult) => {
        const elapsed = performance.now() - startTime;
        console.log(`Pathfinding completed in ${elapsed.toFixed(2)}ms`);
        setResult(pathResult);
        setLoading(false);
      })
      .catch((err) => {
        setError(err);
        setLoading(false);
      });
  };

  return (
    <div style={{ padding: "20px" }}>
      <h2>Transport Switcher Demo</h2>
      <p>Switch between WASM and WebSocket transports seamlessly.</p>

      <div style={{ marginBottom: "20px" }}>
        <label style={{ marginRight: "20px" }}>
          <input
            type="radio"
            value="wasm"
            checked={transport === "wasm"}
            onChange={(e) => setTransport(e.target.value as Transport)}
          />
          WASM (in-browser)
        </label>
        <label>
          <input
            type="radio"
            value="websocket"
            checked={transport === "websocket"}
            onChange={(e) => setTransport(e.target.value as Transport)}
          />
          WebSocket (server)
        </label>
      </div>

      <button
        type="button"
        onClick={runPathfinding}
        disabled={loading}
        style={{
          padding: "10px 20px",
          fontSize: "16px",
          cursor: loading ? "not-allowed" : "pointer",
        }}
      >
        {loading ? "Computing..." : "Find Shortest Path"}
      </button>

      <div style={{ marginTop: "20px" }}>
        <h3>Current Transport: {transport.toUpperCase()}</h3>
        {error && (
          <div style={{ color: "red", padding: "10px", border: "1px solid red" }}>
            <strong>Error:</strong> {error}
          </div>
        )}
        {result && (
          <div
            style={{ color: "green", padding: "10px", border: "1px solid green" }}
          >
            <strong>Path:</strong> [{result.path.join(" → ")}]
            <br />
            <strong>Distance:</strong> {result.distance.toFixed(2)}
          </div>
        )}
      </div>

      <div style={{ marginTop: "20px", fontSize: "14px", color: "#666" }}>
        <h4>How it works:</h4>
        <ul>
          <li>
            <strong>WASM:</strong> Runs pathfinding directly in the browser using
            WebAssembly. No network calls, instant response.
          </li>
          <li>
            <strong>WebSocket:</strong> Sends request to Rust server on port
            10810. Same business logic, different transport.
          </li>
          <li>
            Both use identical API: <code>router.find_shortest_path()</code>
          </li>
        </ul>
      </div>
    </div>
  );
}
