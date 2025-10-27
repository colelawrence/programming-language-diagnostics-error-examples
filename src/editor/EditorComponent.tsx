import Editor from "@monaco-editor/react";
import { useEffect, useState } from "react";
import { useTheme } from "#src/useTheme.tsx";
import type { Router } from "#src/router/types";
import type { AnalyzerDiagnostics, DiagnosticMessage, DiagnosticRich, RichBlock } from "../../dist-types/index";
import { useEditorDiagnostics } from "./useEditorDiagnostics";
import { SplitLayout } from "./SplitLayout";
import type * as Monaco from "monaco-editor";
import { initMermaid, renderMermaidToSvg } from "./mermaid";
import { collapseMarkdown as collapseMd, renderMarkdownToHtml } from "./markdown";
import { useMemo } from "react";
// Track selected diagnostic based on cursor position
function findDiagnosticsAtPosition(diags: AnalyzerDiagnostics, line: number, col0: number) {
  return diags.messages.filter((m) =>
    m.spans.some((s) => line >= s.span.start_line && line <= s.span.end_line && col0 >= s.span.start_column && col0 <= s.span.end_column),
  );
}
// Minimal inline fallback while utilities are added
const collapseMarkdown = (markdown: string, maxLines = 8): string => {
  const lines = markdown.split("\n");
  return lines.length > maxLines ? lines.slice(0, maxLines).join("\n") + "\n…" : markdown;
};

// on global
declare const monaco: typeof Monaco;


interface EditorComponentProps {
  router: Router;
  initialContent?: string;
}

const INITIAL_CODE = `# Welcome to the FFmpeg Command Editor!
# This editor analyzes FFmpeg commands for errors in real-time

# Simple transcode (no errors)
ffmpeg -i input.mp4 output.mp4

# Convert video with specific codec and bitrate
ffmpeg -i video.mov -c:v libx264 -b:v 2M -c:a aac output.mp4

# Try this: Apply video filter to audio-only file (will show error)
ffmpeg -i audio.mp3 -vf scale=1920:1080 output.mp4

# Try this: Use VP9 codec with MP4 container (codec/format incompatibility)
ffmpeg -i input.mp4 -c:v vp9 output.mp4

# Try this: Invalid resolution format (missing 'x')
ffmpeg -i input.mp4 -s 1920 output.mp4`;

export function EditorComponent({ router, initialContent = INITIAL_CODE }: EditorComponentProps) {
  const { theme } = useTheme();
  const [content, setContent] = useState(initialContent);
  const [editorInstance, setEditorInstance] = useState<Monaco.editor.IStandaloneCodeEditor | null>(null);
  const [monacoInstance, setMonacoInstance] = useState<typeof Monaco | null>(null);
  const { diagnostics, isAnalyzing, error, analyzeCode } = useEditorDiagnostics({ router });
  const [cursorInfo, setCursorInfo] = useState<{ line: number; col0: number } | null>(null);

  // Analyze code on content change
  useEffect(() => {
    analyzeCode(content);
  }, [content, analyzeCode]);

  useEffect(() => {
    const monacoTheme = theme === "dark" ? "transparent-dark" : "transparent-light";
    editorInstance?.updateOptions({ theme: monacoTheme });
    monacoInstance?.editor.setTheme(monacoTheme);
  }, [editorInstance, monacoInstance, theme]);

  // Apply markers and decorations to editor when diagnostics change
  useEffect(() => {
    if (!editorInstance || !diagnostics) return;

    const model = editorInstance.getModel();
    if (!model) return;
    // Create Monaco markers for proper hover behavior and problems panel
    const markers = diagnostics.messages.flatMap((msg) =>
      msg.spans.map((span): Monaco.editor.IMarkerData => {
        const isReference = span.role.type === "Reference";
        const isSuggestion = span.role.type === "Suggestion";
        return {
          startLineNumber: span.span.start_line,
          startColumn: span.span.start_column + 1, // Monaco uses 1-based columns
          endLineNumber: span.span.end_line,
          endColumn: span.span.end_column + 1,
          message: `${msg.code}: ${msg.message}`,
          severity: isReference
            ? monaco.MarkerSeverity.Hint
            : isSuggestion
            ? monaco.MarkerSeverity.Info
            : getMonacoSeverity(msg.severity),
          source: "Custom Language Analyzer",
        };
      }),
    );

    // Set markers on the model
    monaco.editor.setModelMarkers(model, "custom-analyzer", markers);

    // Create decorations for custom squiggly styling
    // Note: Markers provide the hover behavior, decorations provide custom visual styling
    const decorations = diagnostics.messages.flatMap((msg) =>
      msg.spans.map((span): Monaco.editor.IModelDeltaDecoration => ({
        range: new monaco.Range(
          span.span.start_line,
          span.span.start_column + 1,
          span.span.end_line,
          span.span.end_column + 1,
        ),
        options: {
          isWholeLine: false,
          // inlineClassName: getSeverityInlineClassName(msg.severity),
          // minimap: {
          //   color: getSeverityColor(msg.severity),
          //   position: 2, // MinimapPosition.Inline
          // },
          // overviewRuler: {
          //   color: getSeverityColor(msg.severity),
          //   position: monaco.editor.OverviewRulerLane.Right,
          // },
        },
      })),
    );

    const decorationIds = editorInstance.createDecorationsCollection(decorations);

    return () => {
      // Clear markers
      monaco.editor.setModelMarkers(model, "custom-analyzer", []);
      // Clear decorations
      decorationIds.clear();
    };
  }, [editorInstance, diagnostics]);

  function handleEditorBeforeMount(monaco: typeof Monaco) {
    initMermaid(theme !== "dark");
    setMonacoInstance(monaco);
    
    // Define transparent dark theme
    monaco.editor.defineTheme("transparent-dark", {
      base: "vs-dark",
      inherit: true,
      rules: [],
      colors: {
        "editor.background": "#00000000",
      },
    });
    
    // Define transparent light theme
    monaco.editor.defineTheme("transparent-light", {
      base: "vs",
      inherit: true,
      rules: [],
      colors: {
        "editor.background": "#00000000",
      },
    });
  }

  function handleEditorDidMount(editor: Monaco.editor.IStandaloneCodeEditor) {
    setEditorInstance(editor);
    // Register hover provider for rich diagnostic previews
    monaco.languages.registerHoverProvider("shell", {
      provideHover(model, position) {
        if (!diagnostics) return null;
        const line = position.lineNumber;
        const col0 = position.column - 1; // convert to 0-based
        const hits = findDiagnosticsAtPosition(diagnostics, line, col0);
        if (hits.length === 0) return null;

        const contents: Monaco.IMarkdownString[] = [];
        for (const d of hits) {
          contents.push({ value: `**${d.code}**: ${d.message}` });
          const blocks = (d.rich as DiagnosticRich | null)?.blocks ?? [];
          for (const block of blocks) {
            if (block.type === "MarkdownGfm") {
              contents.push({ value: collapseMd(block.markdown) });
            }
            if (block.type === "Mermaid") {
              // Render a small preview image; expanded rendering is in the panel
              // We'll embed a placeholder fenced code if rendering fails (handled in util)
              contents.push({ value: "```mermaid\n" + block.mermaid + "\n```" });
            }
          }
        }
        return { contents };
      },
    });

    // Track cursor changes for right-side rich panel
    editor.onDidChangeCursorPosition((e) => {
      setCursorInfo({ line: e.position.lineNumber, col0: e.position.column - 1 });
    });
  }

  return (
    <div className="h-screen bg-background text-text">
      <SplitLayout
        editor={
          <div className="h-full relative">
            <Editor
              height="100%"
              defaultLanguage="shell"
              value={content}
              onChange={(value) => setContent(value || "")}
              beforeMount={handleEditorBeforeMount}
              onMount={handleEditorDidMount}
              theme={theme === "dark" ? "transparent-dark" : "transparent-light"}
              options={{
                fontFamily: "var(--font-mono)",
                fontSize: 15,
                lineHeight: 22,
                minimap: { enabled: false },
                scrollBeyondLastLine: false,
                automaticLayout: true,
                tabSize: 2,
                wordWrap: "on",
                // Disable all language-specific features
                quickSuggestions: false,
                parameterHints: { enabled: false },
                suggestOnTriggerCharacters: false,
                acceptSuggestionOnEnter: "off",
                tabCompletion: "off",
                wordBasedSuggestions: "off",
                // Enable hover to show our diagnostic markers
                hover: { enabled: true, delay: 300 },
              }}
            />
            {isAnalyzing && (
              <div className="absolute top-2 right-2 bg-surface px-3 py-1 rounded border border-border text-text-secondary">
                Analyzing...
              </div>
            )}
          </div>
        }
        diagnosticsPanel={<DiagnosticsPanel diagnostics={diagnostics} error={error} editorInstance={editorInstance} cursorInfo={cursorInfo} />}
        richPanel={<RichPanel diagnostics={diagnostics} cursorInfo={cursorInfo} />}
        initialHorizontalSize={800}
        initialVerticalSize={300}
        minEditorWidth={400}
        minPanelWidth={300}
        minPanelHeight={150}
      />
    </div>
  );
}

interface DiagnosticsPanelProps {
  diagnostics: AnalyzerDiagnostics | null;
  error: string | null;
  editorInstance: Monaco.editor.IStandaloneCodeEditor | null;
  cursorInfo: { line: number; col0: number } | null;
}

function DiagnosticsPanel({ diagnostics, error, editorInstance, cursorInfo }: DiagnosticsPanelProps) {
  const handleDiagnosticClick = (span: DiagnosticMessage["spans"][0]) => {
    if (!editorInstance) return;
    // Set cursor position and selection to the span
    const selection = new monaco.Selection(
      span.span.start_line,
      span.span.start_column + 1, // Monaco uses 1-based columns
      span.span.end_line,
      span.span.end_column + 1,
    );

    editorInstance.setSelection(selection);
    editorInstance.revealRangeInCenter(selection, monaco.editor.ScrollType.Smooth);
    editorInstance.focus();
  };

  const getRoleIndicator = (role: DiagnosticMessage["spans"][0]["role"]) => {
    switch (role.type) {
      case "Reference":
        return "→ ";
      case "Suggestion":
        return "💡 ";
      case "Target":
        return "";
      default:
        return "• ";
    }
  };

  const getSeverityWeight = (severity: DiagnosticMessage["severity"]): number => {
    switch (severity.type) {
      case "Error": return 4;
      case "Warning": return 3;
      case "Info": return 2;
      case "Hint": return 1;
    }
  };

  if (error) {
    return (
      <div className="h-full bg-surface p-4 overflow-auto">
        <div className="text-error font-semibold mb-2">Error</div>
        <div className="text-text-secondary font-mono">{error}</div>
      </div>
    );
  }

  if (!diagnostics || diagnostics.messages.length === 0) {
    return (
      <div className="h-full bg-surface p-4 overflow-auto">
        <div className="text-text-secondary">No errors or warnings</div>
      </div>
    );
  }

  // Sort all diagnostics by severity (Error > Warning > Info > Hint)
  const sortedMessages = [...diagnostics.messages].sort((a, b) => 
    getSeverityWeight(b.severity) - getSeverityWeight(a.severity)
  );

  const errorCount = sortedMessages.filter((m) => m.severity.type === "Error").length;
  const warningCount = sortedMessages.filter((m) => m.severity.type === "Warning").length;

  return (
    <div className="h-full bg-surface overflow-auto">
      <div className="p-4">
        <div className="text-text font-semibold mb-3">
          {errorCount > 0 && `${errorCount} ${errorCount === 1 ? "error" : "errors"}`}
          {errorCount > 0 && warningCount > 0 && ", "}
          {warningCount > 0 && `${warningCount} ${warningCount === 1 ? "warning" : "warnings"}`}
        </div>
        <div className="space-y-3">
          {sortedMessages.map((msg, idx) => (
            <div key={idx} className={`border-l-2 ${getSeverityBorderClass(msg.severity)} pl-3`}>
              <div className="flex items-start gap-2">
                <span className={`${getSeverityTextClass(msg.severity)} font-mono font-semibold`}>{msg.code}</span>
                <div className="flex-1">
                  <div className="text-text">{msg.message}</div>
                  {msg.spans.map((span, spanIdx) => (
                    <button
                      key={spanIdx}
                      onClick={() => handleDiagnosticClick(span)}
                      className="mt-1 text-text-secondary font-mono text-sm hover:text-primary hover:underline cursor-pointer text-left block"
                    >
                      {getRoleIndicator(span.role)}{span.message} — {span.span.start_line}:{span.span.start_column}
                    </button>
                  ))}
                </div>
              </div>
            </div>
          ))}
          <details>
            <summary className="text-text-secondary cursor-pointer">Raw diagnostics</summary>
            <pre>{JSON.stringify(sortedMessages, null, 2)}</pre>
          </details>
        </div>
      </div>
    </div>
  );
}

interface RichPanelProps {
  diagnostics: AnalyzerDiagnostics | null;
  cursorInfo: { line: number; col0: number } | null;
}

function RichPanel({ diagnostics, cursorInfo }: RichPanelProps) {
  const getSeverityWeight = (severity: DiagnosticMessage["severity"]): number => {
    switch (severity.type) {
      case "Error": return 4;
      case "Warning": return 3;
      case "Info": return 2;
      case "Hint": return 1;
    }
  };

  const activeDiagnostics = useMemo(() => {
    if (!diagnostics || !cursorInfo) return [];
    const hits = findDiagnosticsAtPosition(diagnostics, cursorInfo.line, cursorInfo.col0);
    // Sort by severity (Error > Warning > Info > Hint)
    return hits.sort((a, b) => getSeverityWeight(b.severity) - getSeverityWeight(a.severity));
  }, [diagnostics, cursorInfo]);

  if (activeDiagnostics.length === 0) {
    return (
      <div className="h-full bg-surface border-l border-border p-4 overflow-auto text-text-secondary">
        Move cursor into a highlighted span to see details.
      </div>
    );
  }

  return (
    <div className="h-full bg-surface border-l border-border p-4 overflow-auto">
      <div className="space-y-6">
        {activeDiagnostics.map((active, idx) => {
          const blocks = (active.rich as DiagnosticRich | null)?.blocks ?? [];
          return (
            <div key={idx} className={idx > 0 ? "pt-6 border-t border-border" : ""}>
              <div className="text-text font-semibold mb-2">{active.code}: {active.message}</div>
              <div className="space-y-4">
                {blocks.map((b, i) => {
                  if (b.type === "MarkdownGfm") {
                    const html = renderMarkdownToHtml(b.markdown);
                    return <div key={i} className="prose prose-invert" dangerouslySetInnerHTML={{ __html: html }} />;
                  }
                  if (b.type === "Mermaid") {
                    return <MermaidDiagram key={i} code={b.mermaid} />;
                  }
                  return null;
                })}
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
}

function MermaidDiagram({ code }: { code: string }) {
  const [svg, setSvg] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    renderMermaidToSvg(code)
      .then(setSvg)
      .catch((err: unknown) => setError(err instanceof Error ? err.message : String(err)));
  }, [code]);

  if (error) {
    return (
      <div className="bg-surface-elevated p-3 rounded border border-error">
        <div className="text-error font-semibold mb-2">Failed to render diagram</div>
        <pre className="text-text-secondary text-sm overflow-auto">{code}</pre>
      </div>
    );
  }

  if (!svg) {
    return (
      <div className="bg-surface-elevated p-3 rounded border border-border">
        <div className="text-text-secondary">Rendering diagram...</div>
      </div>
    );
  }

  return (
    <div className="bg-surface-elevated p-3 rounded border border-border overflow-auto">
      <div dangerouslySetInnerHTML={{ __html: svg }} />
    </div>
  );
}

function getMonacoSeverity(severity: DiagnosticMessage["severity"]): number {
  switch (severity.type) {
    case "Error":
      return monaco.MarkerSeverity.Error; // 8
    case "Warning":
      return monaco.MarkerSeverity.Warning; // 4
    case "Info":
      return monaco.MarkerSeverity.Info; // 2
    case "Hint":
      return monaco.MarkerSeverity.Hint; // 1
  }
}

function getSeverityBorderClass(severity: DiagnosticMessage["severity"]): string {
  switch (severity.type) {
    case "Error":
      return "border-error";
    case "Warning":
      return "border-warning";
    case "Info":
      return "border-accent";
    case "Hint":
      return "border-border";
  }
}

function getSeverityTextClass(severity: DiagnosticMessage["severity"]): string {
  switch (severity.type) {
    case "Error":
      return "text-error";
    case "Warning":
      return "text-warning";
    case "Info":
      return "text-accent";
    case "Hint":
      return "text-text-secondary";
  }
}

