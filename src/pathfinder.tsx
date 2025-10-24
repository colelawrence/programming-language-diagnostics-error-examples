import * as d3 from "d3";
import { useEffect, useRef, useState } from "react";
import {
  createRouter,
  createWasmAdaptor,
  createWebSocketAdaptor,
  type Router,
} from "./router";

type Point = { x: number; y: number };
type Edge = { from: number; to: number };
type Transport = "wasm" | "websocket";

export function PathfinderDemo() {
  const svgRef = useRef<SVGSVGElement>(null);
  const [router, setRouter] = useState<Router | null>(null);
  const [transport, setTransport] = useState<Transport>(() => {
    const saved = localStorage.getItem("pathfinder-transport");
    return (saved as Transport) || "wasm";
  });
  const [connectionStatus, setConnectionStatus] = useState<string>("initializing");
  const [points, setPoints] = useState<Point[]>([
    { x: 100, y: 100 },
    { x: 300, y: 100 },
    { x: 500, y: 100 },
    { x: 100, y: 300 },
    { x: 300, y: 300 },
    { x: 500, y: 300 },
  ]);
  const [edges] = useState<Edge[]>([
    { from: 0, to: 1 },
    { from: 1, to: 2 },
    { from: 0, to: 3 },
    { from: 1, to: 4 },
    { from: 2, to: 5 },
    { from: 3, to: 4 },
    { from: 4, to: 5 },
  ]);
  const [path, setPath] = useState<number[]>([]);
  const [startIdx, setStartIdx] = useState(0);
  const [endIdx, setEndIdx] = useState(5);
  const [metrics, setMetrics] = useState<{
    node_count: number;
    edge_count: number;
    total_edge_length: number;
    avg_edge_length: number;
  } | null>(null);

  useEffect(() => {
    // Save transport preference
    localStorage.setItem("pathfinder-transport", transport);

    // Create router with selected transport
    setConnectionStatus("connecting...");
    const newRouter = createRouter({
      adaptor:
        transport === "wasm"
          ? createWasmAdaptor()
          : createWebSocketAdaptor({
              url: "ws://localhost:10810",
              autoReconnect: true,
              reconnectDelay: 1000,
            }),
    });
    
    setRouter(newRouter);
    setConnectionStatus("ready");

    return () => {
      newRouter.dispose();
    };
  }, [transport]);

  const handleTransportChange = (newTransport: Transport) => {
    setTransport(newTransport);
    setPath([]); // Clear current path when switching
  };

  useEffect(() => {
    if (!router) return;

    const controller = router
      .find_shortest_path({
        points,
        edges,
        start_idx: startIdx,
        end_idx: endIdx,
      })
      .each(
        (result) => {
          setPath(result.path);
        },
        { signal: AbortSignal.timeout(5000) },
      );

    return () => controller.abort();
  }, [router, points, edges, startIdx, endIdx]);

  useEffect(() => {
    if (!router) return;

    const controller = router
      .compute_graph_metrics({
        points,
        edges,
      })
      .each(
        (result) => {
          setMetrics(result);
        },
        { signal: AbortSignal.timeout(5000) },
      );

    return () => controller.abort();
  }, [router, points, edges]);

  useEffect(() => {
    if (!svgRef.current) return;

    const svg = d3.select(svgRef.current);
    svg.selectAll("*").remove();

    const width = 600;
    const height = 400;

    svg.attr("width", width).attr("height", height);

    svg
      .selectAll("line")
      .data(edges)
      .enter()
      .append("line")
      .attr("x1", (d) => points[d.from].x)
      .attr("y1", (d) => points[d.from].y)
      .attr("x2", (d) => points[d.to].x)
      .attr("y2", (d) => points[d.to].y)
      .attr("stroke", "var(--color-border)")
      .attr("stroke-width", 2);

    if (path.length > 1) {
      for (let i = 0; i < path.length - 1; i++) {
        svg
          .append("line")
          .attr("x1", points[path[i]].x)
          .attr("y1", points[path[i]].y)
          .attr("x2", points[path[i + 1]].x)
          .attr("y2", points[path[i + 1]].y)
          .attr("stroke", "var(--color-terminal-magenta)")
          .attr("stroke-width", 3);
      }
    }

    const circles = svg
      .selectAll("circle")
      .data(points.map((p, i) => ({ ...p, index: i })))
      .enter()
      .append("circle")
      .attr("cx", (d) => d.x)
      .attr("cy", (d) => d.y)
      .attr("r", 8)
      .attr("fill", (d) => {
        if (d.index === startIdx) return "var(--color-primary)";
        if (d.index === endIdx) return "var(--color-error)";
        if (path.includes(d.index)) return "var(--color-warning)";
        return "var(--color-accent)";
      })
      .attr("stroke", "var(--color-text)")
      .attr("stroke-width", 2)
      .style("cursor", "pointer");

    circles.call(
      d3
        .drag<SVGCircleElement, Point & { index: number }>()
        .on("drag", (event, d) => {
          const newPoints = [...points];
          newPoints[d.index] = { x: event.x, y: event.y };
          setPoints(newPoints);
        }) as any,
    );

    svg
      .selectAll("text")
      .data(points)
      .enter()
      .append("text")
      .attr("x", (d) => d.x)
      .attr("y", (d) => d.y - 12)
      .attr("text-anchor", "middle")
      .attr("fill", "var(--color-text)")
      .attr("font-size", "var(--text-label)")
      .attr("font-family", "var(--font-mono)")
      .text((_, i) => i);
  }, [points, edges, path, startIdx, endIdx]);

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between pb-4 border-b border-border">
        <div className="flex items-center gap-4">
          <label className="flex items-center gap-2">
            <span className="text-accent">start:</span>
            <input
              type="number"
              min="0"
              max={points.length - 1}
              value={startIdx}
              onChange={(e) => setStartIdx(Number(e.target.value))}
              className="w-16 px-2 py-1 bg-surface border border-border text-text focus:outline-none focus:border-border-focus"
            />
          </label>
          <span className="text-text-tertiary">→</span>
          <label className="flex items-center gap-2">
            <span className="text-accent">end:</span>
            <input
              type="number"
              min="0"
              max={points.length - 1}
              value={endIdx}
              onChange={(e) => setEndIdx(Number(e.target.value))}
              className="w-16 px-2 py-1 bg-surface border border-border text-text focus:outline-none focus:border-border-focus"
            />
          </label>
        </div>

        {/* Transport Selector */}
        <div className="flex items-center gap-3 px-3 py-2 bg-surface border border-border">
          <span className="text-text-secondary text-label">transport:</span>
          <label className="flex items-center gap-2 cursor-pointer">
            <input
              type="radio"
              value="wasm"
              checked={transport === "wasm"}
              onChange={(e) => handleTransportChange(e.target.value as Transport)}
              className="cursor-pointer"
            />
            <span className="text-text">🦀 wasm</span>
          </label>
          <label className="flex items-center gap-2 cursor-pointer">
            <input
              type="radio"
              value="websocket"
              checked={transport === "websocket"}
              onChange={(e) => handleTransportChange(e.target.value as Transport)}
              className="cursor-pointer"
            />
            <span className="text-text">🌐 ws</span>
          </label>
          <span 
            className="text-label"
            style={{ 
              color: connectionStatus === "ready" 
                ? "var(--color-primary)" 
                : "var(--color-warning)" 
            }}
          >
            {connectionStatus === "ready" ? "●" : "○"}
          </span>
        </div>
      </div>

      {/* Metrics banner */}
      <div className="grid grid-cols-2 gap-4">
        <div 
          className="px-3 py-2 text-label border border-border"
          style={{ 
            background: transport === "wasm" 
              ? "rgba(46, 125, 50, 0.1)" 
              : "rgba(38, 166, 154, 0.1)"
          }}
        >
          <strong className="text-text">
            {transport === "wasm" ? "in-browser" : "server-side"}:
          </strong>{" "}
          <span className="text-text-secondary">
            {transport === "wasm" 
              ? "pathfinding runs locally via WebAssembly" 
              : "pathfinding via WebSocket (requires server on :10810)"
            }
          </span>
        </div>
        
        {/* Graph metrics */}
        {metrics && (
          <div className="px-3 py-2 text-label border border-border bg-surface">
            <strong className="text-accent">graph metrics:</strong>{" "}
            <span className="text-text-secondary">
              {metrics.node_count} nodes • {metrics.edge_count} edges • 
              avg length: {metrics.avg_edge_length.toFixed(1)}px
            </span>
          </div>
        )}
      </div>

      <p className="text-text-secondary text-label">
        drag nodes • <span className="text-primary">green</span>=start •{" "}
        <span className="text-error">red</span>=end •{" "}
        <span className="text-warning">orange</span>=path
      </p>
      <svg ref={svgRef} className="border border-border bg-surface" />
    </div>
  );
}
