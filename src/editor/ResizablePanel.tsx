import { useState, useRef, useEffect, type ReactNode } from "react";

interface ResizablePanelProps {
  children: [ReactNode, ReactNode];
  initialTopHeight?: number;
  minTopHeight?: number;
  minBottomHeight?: number;
}

export function ResizablePanel({
  children,
  initialTopHeight = 400,
  minTopHeight = 200,
  minBottomHeight = 150,
}: ResizablePanelProps) {
  const [topHeight, setTopHeight] = useState(initialTopHeight);
  const [isResizing, setIsResizing] = useState(false);
  const containerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!isResizing) return;

    const handleMouseMove = (e: MouseEvent) => {
      if (!containerRef.current) return;

      const containerRect = containerRef.current.getBoundingClientRect();
      const containerHeight = containerRect.height;
      const newTopHeight = e.clientY - containerRect.top;

      // Clamp height to min/max constraints
      const clampedHeight = Math.max(
        minTopHeight,
        Math.min(containerHeight - minBottomHeight, newTopHeight),
      );

      setTopHeight(clampedHeight);
    };

    const handleMouseUp = () => {
      setIsResizing(false);
    };

    document.addEventListener("mousemove", handleMouseMove);
    document.addEventListener("mouseup", handleMouseUp);

    return () => {
      document.removeEventListener("mousemove", handleMouseMove);
      document.removeEventListener("mouseup", handleMouseUp);
    };
  }, [isResizing, minTopHeight, minBottomHeight]);

  const handleMouseDown = (e: React.MouseEvent) => {
    e.preventDefault();
    setIsResizing(true);
  };

  return (
    <div ref={containerRef} className="flex flex-col h-full w-full">
      {/* Top panel */}
      <div style={{ height: topHeight }} className="overflow-hidden">
        {children[0]}
      </div>

      {/* Resize handle */}
      <div
        onMouseDown={handleMouseDown}
        className={`
          h-1 bg-border cursor-row-resize flex-shrink-0
          hover:bg-primary hover:h-1.5 transition-all
          ${isResizing ? "bg-primary h-1.5" : ""}
        `}
        role="separator"
        aria-orientation="horizontal"
        aria-valuenow={topHeight}
        aria-label="Resize panels"
      />

      {/* Bottom panel */}
      <div className="flex-1 overflow-hidden">{children[1]}</div>
    </div>
  );
}

