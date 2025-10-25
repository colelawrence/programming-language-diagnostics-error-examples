import { useState, useEffect, useMemo } from "react";
import { EditorComponent } from "#src/editor/EditorComponent";
import { createRouter } from "#src/router";
import { wasmAdaptor } from "#src/router/wasmAdaptor";
import { createWebSocketAdaptor } from "#src/router/websocketAdaptor";
import type { Router } from "#src/router/types";

type Transport = "wasm" | "websocket";

export function EditorTransportExample() {
  const [transport, setTransport] = useState<Transport>("wasm");
  const [router, setRouter] = useState<Router | null>(null);

  useEffect(() => {
    // Create router based on selected transport
    const newRouter = createRouter({
      adaptor: transport === "wasm" ? wasmAdaptor : createWebSocketAdaptor({ url: "ws://localhost:10810" }),
    });

    setRouter(newRouter);

    return () => {
      newRouter.dispose();
    };
  }, [transport]);

  if (!router) {
    return <div className="p-4 text-text">Initializing router...</div>;
  }

  return (
    <div className="flex flex-col h-screen bg-background">
      {/* Transport Selector */}
      <div className="border-b border-border bg-surface px-4 py-3 flex items-center gap-4">
        <span className="text-text font-semibold">Transport:</span>
        <label className="flex items-center gap-2 cursor-pointer">
          <input
            type="radio"
            value="wasm"
            checked={transport === "wasm"}
            onChange={(e) => setTransport(e.target.value as Transport)}
            className="cursor-pointer"
          />
          <span className="text-text">WASM (in-browser)</span>
        </label>
        <label className="flex items-center gap-2 cursor-pointer">
          <input
            type="radio"
            value="websocket"
            checked={transport === "websocket"}
            onChange={(e) => setTransport(e.target.value as Transport)}
            className="cursor-pointer"
          />
          <span className="text-text">WebSocket (server)</span>
        </label>
        <div className="ml-auto text-text-secondary text-sm">
          {transport === "wasm"
            ? "Running code analysis in your browser"
            : "Connected to Rust server on port 10810"}
        </div>
      </div>

      {/* Editor */}
      <div className="flex-1">
        <EditorComponent router={router} />
      </div>
    </div>
  );
}

