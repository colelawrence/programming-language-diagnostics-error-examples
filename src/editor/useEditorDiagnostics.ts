import { useEffect, useState, useRef, useCallback } from "react";
import type { Router } from "#src/router/types";
import type { AnalyzerDiagnostics, AnalyzeCodeParams } from "../../dist-types/index";

export interface UseEditorDiagnosticsOptions {
  router: Router;
  debounceMs?: number;
  enabled?: boolean;
}

export function useEditorDiagnostics(options: UseEditorDiagnosticsOptions) {
  const { router, debounceMs = 500, enabled = true } = options;
  const [diagnostics, setDiagnostics] = useState<AnalyzerDiagnostics | null>(null);
  const [isAnalyzing, setIsAnalyzing] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const debounceTimer = useRef<Timer | null>(null);
  const abortController = useRef<AbortController | null>(null);

  const analyzeCode = useCallback(
    (content: string, filePath?: string) => {
      if (!enabled) return;

      // Cancel previous analysis
      if (abortController.current) {
        abortController.current.abort();
      }

      // Clear existing debounce timer
      if (debounceTimer.current) {
        clearTimeout(debounceTimer.current);
      }

      // Set up new debounced analysis
      debounceTimer.current = setTimeout(async () => {
        setIsAnalyzing(true);
        setError(null);

        const controller = new AbortController();
        abortController.current = controller;

        try {
          // Split content into lines and analyze each FFmpeg command separately
          const lines = content.split("\n");
          const analyzePromises: Promise<AnalyzerDiagnostics>[] = [];

          for (let lineIndex = 0; lineIndex < lines.length; lineIndex++) {
            const line = lines[lineIndex];
            const trimmedLine = line.trim();

            // Skip empty lines and comments
            if (!trimmedLine || trimmedLine.startsWith("#")) {
              continue;
            }

            // Find the column offset (where non-whitespace starts)
            const columnOffset = line.search(/\S/);
            const actualColumnOffset = columnOffset >= 0 ? columnOffset : 0;

            // Analyze this line with its offsets
            // Monaco Editor uses 1-based line numbers, so add 1 to the 0-based lineIndex
            const params: AnalyzeCodeParams = {
              content: trimmedLine,
              file_path: filePath || null,
              line_offset: lineIndex + 1,
              column_offset: actualColumnOffset,
            };

            analyzePromises.push(router.analyze_code(params).first());
          }

          // Wait for all analyses to complete
          const results = await Promise.all(analyzePromises);

          if (!controller.signal.aborted) {
            // Merge all diagnostics
            const allMessages = results.flatMap((result) => result.messages);
            setDiagnostics({ messages: allMessages });
            setIsAnalyzing(false);
          }
        } catch (err) {
          if (!controller.signal.aborted) {
            setError(err instanceof Error ? err.message : String(err));
            setIsAnalyzing(false);
          }
        }
      }, debounceMs);
    },
    [router, debounceMs, enabled],
  );

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      if (debounceTimer.current) {
        clearTimeout(debounceTimer.current);
      }
      if (abortController.current) {
        abortController.current.abort();
      }
    };
  }, []);

  return {
    diagnostics,
    isAnalyzing,
    error,
    analyzeCode,
  };
}

