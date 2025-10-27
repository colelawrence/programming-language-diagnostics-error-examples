import { useState, useRef, useEffect, type ReactNode } from "react";

interface ResizablePanelProps {
  children: [ReactNode, ReactNode];
  direction?: "vertical" | "horizontal";
  initialSize?: number;
  minFirstSize?: number;
  minSecondSize?: number;
}

export function ResizablePanel({
  children,
  direction = "vertical",
  initialSize = 400,
  minFirstSize = 200,
  minSecondSize = 150,
}: ResizablePanelProps) {
  const [firstSize, setFirstSize] = useState(initialSize);
  const [isResizing, setIsResizing] = useState(false);
  const containerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!isResizing) return;

    const handleMouseMove = (e: MouseEvent) => {
      if (!containerRef.current) return;

      const containerRect = containerRef.current.getBoundingClientRect();
      const containerSize = direction === "vertical" ? containerRect.height : containerRect.width;
      const mousePos = direction === "vertical" 
        ? e.clientY - containerRect.top 
        : e.clientX - containerRect.left;

      // Clamp size to min/max constraints
      const clampedSize = Math.max(
        minFirstSize,
        Math.min(containerSize - minSecondSize, mousePos),
      );

      setFirstSize(clampedSize);
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
  }, [isResizing, minFirstSize, minSecondSize, direction]);

  const handleMouseDown = (e: React.MouseEvent) => {
    e.preventDefault();
    setIsResizing(true);
  };

  const isVertical = direction === "vertical";

  return (
    <div ref={containerRef} className={`flex ${isVertical ? "flex-col" : "flex-row"} h-full w-full`}>
      {/* First panel */}
      <div 
        style={isVertical ? { height: firstSize } : { width: firstSize }} 
        className="overflow-hidden"
      >
        {children[0]}
      </div>

      {/* Resize handle */}
      <div
        onMouseDown={handleMouseDown}
        className={`
          ${isVertical ? "h-1 w-full cursor-row-resize" : "w-1 h-full cursor-col-resize"}
          bg-border flex-shrink-0
          ${isVertical ? "hover:h-1.5" : "hover:w-1.5"} hover:bg-primary transition-all
          ${isResizing ? `bg-primary ${isVertical ? "h-1.5" : "w-1.5"}` : ""}
        `}
        role="separator"
        aria-orientation={isVertical ? "horizontal" : "vertical"}
        aria-valuenow={firstSize}
        aria-label="Resize panels"
      />

      {/* Second panel */}
      <div className="flex-1 overflow-hidden">{children[1]}</div>
    </div>
  );
}

