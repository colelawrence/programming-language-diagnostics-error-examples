import { useEffect, useState, useRef, useCallback } from "react";
import type { Router } from "#src/router/types";
import type { AnalyzerDiagnostics } from "../../dist-types/index";

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
      debounceTimer.current = setTimeout(() => {
        setIsAnalyzing(true);
        setError(null);

        const controller = new AbortController();
        abortController.current = controller;

        router
          .analyze_code({
            content,
            file_path: filePath || null,
          })
          .first({ signal: controller.signal })
          .then((result) => {
            if (!controller.signal.aborted) {
              setDiagnostics(result);
              setIsAnalyzing(false);
            }
          })
          .catch((err) => {
            if (!controller.signal.aborted) {
              setError(err instanceof Error ? err.message : String(err));
              setIsAnalyzing(false);
            }
          });
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

