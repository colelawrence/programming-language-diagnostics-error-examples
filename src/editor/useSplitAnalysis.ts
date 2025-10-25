import { useCallback } from "react";
import type { Router } from "#src/router/types";
import type { AnalyzerDiagnostics, AnalyzeCodeParams } from "../../dist-types/index";

/**
 * Hook to split multi-line input and analyze each FFmpeg command separately with proper offsets
 */
export function useSplitAnalysis(router: Router) {
  const analyzeWithOffsets = useCallback(
    async (content: string): Promise<AnalyzerDiagnostics> => {
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
          file_path: null,
          line_offset: lineIndex + 1,
          column_offset: actualColumnOffset,
        };

        analyzePromises.push(router.analyze_code(params).first());
      }

      // Wait for all analyses to complete
      const results = await Promise.all(analyzePromises);

      // Merge all diagnostics
      const allMessages = results.flatMap((result) => result.messages);

      return {
        messages: allMessages,
      };
    },
    [router]
  );

  return { analyzeWithOffsets };
}

