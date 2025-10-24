import { useState } from "react";
import { createRouter, createWebSocketAdaptor } from "../router";

export function WebSocketExample() {
  const [connected, setConnected] = useState(false);
  const [result, setResult] = useState<string>("");

  const testWebSocket = async () => {
    try {
      const router = createRouter({
        adaptor: createWebSocketAdaptor({
          url: "ws://localhost:10810",
        }),
      });

      setConnected(true);
      setResult("Finding path via WebSocket...");

      router
        .find_shortest_path({
          points: [
            { x: 0, y: 0 },
            { x: 1, y: 0 },
            { x: 2, y: 0 },
          ],
          edges: [
            { from: 0, to: 1 },
            { from: 1, to: 2 },
          ],
          start_idx: 0,
          end_idx: 2,
        })
        .first()
        .then((pathResult) => {
          setResult(
            `Path found: [${pathResult.path.join(", ")}], Distance: ${pathResult.distance}`,
          );
          console.log("Request complete");
        })
        .catch((error) => {
          setResult(`Error: ${error}`);
        });
    } catch (error) {
      setResult(`Connection failed: ${error}`);
    }
  };

  return (
    <div style={{ padding: "20px" }}>
      <h2>WebSocket Transport Example</h2>
      <p>This example uses WebSocket to communicate with the Rust server.</p>
      <button type="button" onClick={testWebSocket}>
        Test WebSocket
      </button>
      <p>Status: {connected ? "Connected" : "Disconnected"}</p>
      <p>{result}</p>
    </div>
  );
}
