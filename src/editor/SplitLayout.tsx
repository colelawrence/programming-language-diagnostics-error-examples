import { useState, useRef, useEffect, type ReactNode } from "react";

interface SplitLayoutProps {
  editor: ReactNode;
  diagnosticsPanel: ReactNode;
  richPanel: ReactNode;
  initialHorizontalSize?: number;
  initialVerticalSize?: number;
  minEditorWidth?: number;
  minPanelWidth?: number;
  minPanelHeight?: number;
}

export function SplitLayout({
  editor,
  diagnosticsPanel,
  richPanel,
  initialHorizontalSize = 800,
  initialVerticalSize = 300,
  minEditorWidth = 400,
  minPanelWidth = 300,
  minPanelHeight = 150,
}: SplitLayoutProps) {
  const [editorWidth, setEditorWidth] = useState(initialHorizontalSize);
  const [diagnosticsHeight, setDiagnosticsHeight] = useState(initialVerticalSize);
  const [resizeMode, setResizeMode] = useState<"horizontal" | "vertical" | "both" | null>(null);
  const containerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!resizeMode) return;

    const handleMouseMove = (e: MouseEvent) => {
      if (!containerRef.current) return;

      const containerRect = containerRef.current.getBoundingClientRect();

      if (resizeMode === "horizontal" || resizeMode === "both") {
        const newWidth = e.clientX - containerRect.left;
        const clampedWidth = Math.max(
          minEditorWidth,
          Math.min(containerRect.width - minPanelWidth, newWidth),
        );
        setEditorWidth(clampedWidth);
      }

      if (resizeMode === "vertical" || resizeMode === "both") {
        const panelTop = containerRect.top;
        const newHeight = e.clientY - panelTop;
        const clampedHeight = Math.max(
          minPanelHeight,
          Math.min(containerRect.height - minPanelHeight, newHeight),
        );
        setDiagnosticsHeight(clampedHeight);
      }
    };

    const handleMouseUp = () => {
      setResizeMode(null);
    };

    document.addEventListener("mousemove", handleMouseMove);
    document.addEventListener("mouseup", handleMouseUp);

    return () => {
      document.removeEventListener("mousemove", handleMouseMove);
      document.removeEventListener("mouseup", handleMouseUp);
    };
  }, [resizeMode, minEditorWidth, minPanelWidth, minPanelHeight]);

  return (
    <div ref={containerRef} className="flex h-full w-full">
      {/* Editor panel */}
      <div style={{ width: editorWidth }} className="h-full overflow-hidden">
        {editor}
      </div>

      {/* Vertical resize handle */}
      <div
        onMouseDown={(e) => {
          e.preventDefault();
          setResizeMode("horizontal");
        }}
        className={`
          w-1 h-full cursor-col-resize flex-shrink-0 bg-border
          hover:w-1.5 hover:bg-primary transition-all
          ${resizeMode === "horizontal" ? "bg-primary w-1.5" : ""}
        `}
        role="separator"
        aria-orientation="vertical"
      />

      {/* Right panel container */}
      <div className="flex-1 flex flex-col h-full overflow-hidden">
        {/* Diagnostics panel */}
        <div style={{ height: diagnosticsHeight }} className="w-full overflow-hidden">
          {diagnosticsPanel}
        </div>

        {/* Horizontal resize handle with intersection circle */}
        <div className="relative w-full h-1 flex-shrink-0">
          <div
            onMouseDown={(e) => {
              e.preventDefault();
              setResizeMode("vertical");
            }}
            className={`
              w-full h-full cursor-row-resize bg-border
              hover:h-1.5 hover:bg-primary transition-all
              ${resizeMode === "vertical" ? "bg-primary h-1.5" : ""}
            `}
            role="separator"
            aria-orientation="horizontal"
          />
          
          {/* Circular intersection handle */}
          <div
            onMouseDown={(e) => {
              e.preventDefault();
              setResizeMode("both");
            }}
            className={`
              absolute left-0 top-1/2 -translate-x-1/2 -translate-y-1/2
              w-3 h-3 rounded-full cursor-move
              bg-border border border-background
              hover:bg-primary hover:scale-150
              transition-all z-10
              ${resizeMode === "both" ? "bg-primary scale-150" : ""}
            `}
            role="separator"
            aria-label="Resize both panels"
          />
        </div>

        {/* Rich panel */}
        <div className="flex-1 overflow-hidden">
          {richPanel}
        </div>
      </div>
    </div>
  );
}
