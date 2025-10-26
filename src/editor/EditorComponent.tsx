import Editor from "@monaco-editor/react";
import { useEffect, useState } from "react";
import { useTheme } from "#src/useTheme";
import type { Router } from "#src/router/types";
import type { AnalyzerDiagnostics, DiagnosticMessage, DiagnosticRich, RichBlock } from "../../dist-types/index";
import { useEditorDiagnostics } from "./useEditorDiagnostics";
import { ResizablePanel } from "./ResizablePanel";
import type * as Monaco from "monaco-editor";
import { initMermaid, renderMermaidToDataUri } from "./mermaid";
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
# ffmpeg -i audio.mp3 -vf scale=1920:1080 output.mp4

# Try this: Use VP9 codec with MP4 container (codec/format incompatibility)
# ffmpeg -i input.mp4 -c:v vp9 output.mp4

# Try this: Invalid resolution format (missing 'x')
# ffmpeg -i input.mp4 -s 1920 output.mp4`;

export function EditorComponent({ router, initialContent = INITIAL_CODE }: EditorComponentProps) {
  const { theme } = useTheme();
  const [content, setContent] = useState(initialContent);
  const [editorInstance, setEditorInstance] = useState<Monaco.editor.IStandaloneCodeEditor | null>(null);
  const { diagnostics, isAnalyzing, error, analyzeCode } = useEditorDiagnostics({ router });
  const [cursorInfo, setCursorInfo] = useState<{ line: number; col0: number } | null>(null);

  // Analyze code on content change
  useEffect(() => {
    analyzeCode(content);
  }, [content, analyzeCode]);

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
    // Robust color resolvers that work across browsers (including iOS Safari)
    const rgbStringToHex = (color: string): string => {
      // Expect formats like: rgb(r, g, b) or rgba(r, g, b, a)
      const m = color.match(/rgba?\((\d+),\s*(\d+),\s*(\d+)/i);
      if (!m) return "#000000";
      const [r, g, b] = [Number(m[1]), Number(m[2]), Number(m[3])];
      const toHex = (n: number) => Math.max(0, Math.min(255, n)).toString(16).padStart(2, "0");
      return `#${toHex(r)}${toHex(g)}${toHex(b)}`;
    };

    // Resolve a CSS var by applying it to a probe element and reading computed color
    const resolveCssVarToHex = (varName: string, isLightTheme: boolean): string => {
      const root = document.documentElement;
      const originalTheme = root.getAttribute("data-theme");

      if (isLightTheme) root.setAttribute("data-theme", "light");
      else root.removeAttribute("data-theme");

      const probe = document.createElement("div");
      probe.style.position = "absolute";
      probe.style.left = "-9999px";
      probe.style.top = "0";
      probe.style.width = "0";
      probe.style.height = "0";
      probe.style.pointerEvents = "none";
      probe.style.color = `var(${varName})`;
      document.body.appendChild(probe);
      const color = getComputedStyle(probe).color || "";
      probe.remove();

      if (originalTheme === "light") root.setAttribute("data-theme", "light");
      else root.removeAttribute("data-theme");

      return rgbStringToHex(color);
    };

    // Define custom dark theme with dynamically converted colors
    monaco.editor.defineTheme("terminal-dark", {
      base: "vs-dark",
      inherit: true,
      rules: [],
      colors: {
        "editor.background": resolveCssVarToHex("--color-terminal-black", false),
        "editor.foreground": resolveCssVarToHex("--color-terminal-text", false),
        "editor.lineHighlightBackground": resolveCssVarToHex("--color-terminal-gray", false),
        "editorLineNumber.foreground": resolveCssVarToHex("--color-terminal-text-dimmer", false),
        "editorLineNumber.activeForeground": resolveCssVarToHex("--color-terminal-text-dim", false),
        "editor.selectionBackground": resolveCssVarToHex("--color-terminal-gray-light", false),
        "editor.inactiveSelectionBackground": resolveCssVarToHex("--color-terminal-gray", false),
        "editorCursor.foreground": resolveCssVarToHex("--color-terminal-green", false),
        "editorWhitespace.foreground": resolveCssVarToHex("--color-terminal-text-dimmer", false),
        "editorIndentGuide.background": resolveCssVarToHex("--color-terminal-border", false),
        "editorIndentGuide.activeBackground": resolveCssVarToHex("--color-terminal-border-bright", false),
        "editorWidget.background": resolveCssVarToHex("--color-terminal-gray", false),
        "editorWidget.border": resolveCssVarToHex("--color-terminal-border", false),
        "editorHoverWidget.background": resolveCssVarToHex("--color-terminal-gray-light", false),
        "editorHoverWidget.border": resolveCssVarToHex("--color-terminal-border-bright", false),
        "editorSuggestWidget.background": resolveCssVarToHex("--color-terminal-gray", false),
        "editorSuggestWidget.border": resolveCssVarToHex("--color-terminal-border", false),
        "editorSuggestWidget.selectedBackground": resolveCssVarToHex("--color-terminal-gray-light", false),
      },
    });

    // Define custom light theme with dynamically converted colors
    monaco.editor.defineTheme("terminal-light", {
      base: "vs",
      inherit: true,
      rules: [],
      colors: {
        "editor.background": resolveCssVarToHex("--color-terminal-black", true),
        "editor.foreground": resolveCssVarToHex("--color-terminal-text", true),
        "editor.lineHighlightBackground": resolveCssVarToHex("--color-terminal-gray", true),
        "editorLineNumber.foreground": resolveCssVarToHex("--color-terminal-text-dimmer", true),
        "editorLineNumber.activeForeground": resolveCssVarToHex("--color-terminal-text-dim", true),
        "editor.selectionBackground": resolveCssVarToHex("--color-terminal-gray-light", true),
        "editor.inactiveSelectionBackground": resolveCssVarToHex("--color-terminal-gray", true),
        "editorCursor.foreground": resolveCssVarToHex("--color-terminal-green", true),
        "editorWhitespace.foreground": resolveCssVarToHex("--color-terminal-text-dimmer", true),
        "editorIndentGuide.background": resolveCssVarToHex("--color-terminal-border", true),
        "editorIndentGuide.activeBackground": resolveCssVarToHex("--color-terminal-border-bright", true),
        "editorWidget.background": resolveCssVarToHex("--color-terminal-gray", true),
        "editorWidget.border": resolveCssVarToHex("--color-terminal-border", true),
        "editorHoverWidget.background": resolveCssVarToHex("--color-terminal-gray-light", true),
        "editorHoverWidget.border": resolveCssVarToHex("--color-terminal-border-bright", true),
        "editorSuggestWidget.background": resolveCssVarToHex("--color-terminal-gray", true),
        "editorSuggestWidget.border": resolveCssVarToHex("--color-terminal-border", true),
        "editorSuggestWidget.selectedBackground": resolveCssVarToHex("--color-terminal-gray-light", true),
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
      <ResizablePanel initialTopHeight={500} minTopHeight={200} minBottomHeight={150}>
        {/* Editor */}
        <div className="h-full relative">
          <Editor
            height="100%"
            defaultLanguage="shell"
            value={content}
            onChange={(value) => setContent(value || "")}
            beforeMount={handleEditorBeforeMount}
            onMount={handleEditorDidMount}
            theme={theme === "dark" ? "terminal-dark" : "terminal-light"}
            options={{
              fontFamily: "var(--font-mono)",
              fontSize: 15,
              lineHeight: 22,
              minimap: { enabled: true },
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

        {/* Diagnostics Panel and Rich Panel */}
        <div className="grid grid-cols-2 h-full">
          <DiagnosticsPanel diagnostics={diagnostics} error={error} editorInstance={editorInstance} />
          <RichPanel diagnostics={diagnostics} cursorInfo={cursorInfo} />
        </div>
      </ResizablePanel>
    </div>
  );
}

interface DiagnosticsPanelProps {
  diagnostics: AnalyzerDiagnostics | null;
  error: string | null;
  editorInstance: Monaco.editor.IStandaloneCodeEditor | null;
}

function DiagnosticsPanel({ diagnostics, error, editorInstance }: DiagnosticsPanelProps) {
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

  const errorCount = diagnostics.messages.filter((m) => m.severity.type === "Error").length;
  const warningCount = diagnostics.messages.filter((m) => m.severity.type === "Warning").length;

  return (
    <div className="h-full bg-surface overflow-auto">
      <div className="p-4">
        <div className="text-text font-semibold mb-3">
          {errorCount > 0 && `${errorCount} ${errorCount === 1 ? "error" : "errors"}`}
          {errorCount > 0 && warningCount > 0 && ", "}
          {warningCount > 0 && `${warningCount} ${warningCount === 1 ? "warning" : "warnings"}`}
        </div>
        <div className="space-y-3">
          {diagnostics.messages.map((msg, idx) => (
            <div key={idx} className={`border-l-2 ${getSeverityBorderClass(msg.severity)} pl-3`}>
              <div className="flex items-start gap-2">
                <span className={`${getSeverityTextClass(msg.severity)} font-mono font-semibold`}>{msg.code}</span>
                <div className="flex-1">
                  <div className="text-text">{msg.message}</div>
                  {msg.spans.filter((s) => s.role.type !== "Target").map((span, spanIdx) => (
                    <button
                      key={spanIdx}
                      onClick={() => handleDiagnosticClick(span)}
                      className="mt-1 text-text-secondary font-mono text-sm hover:text-primary hover:underline cursor-pointer text-left block"
                    >
                      {span.message} — Line {span.span.start_line}, Col {span.span.start_column}
                    </button>
                  ))}
                </div>
              </div>
            </div>
          ))}
          <pre>{JSON.stringify(diagnostics, null, 2)}</pre>
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
  const active = useMemo(() => {
    if (!diagnostics || !cursorInfo) return null;
    const hits = findDiagnosticsAtPosition(diagnostics, cursorInfo.line, cursorInfo.col0);
    return hits[0] || null;
  }, [diagnostics, cursorInfo]);

  if (!active) {
    return (
      <div className="h-full bg-surface border-l border-border p-4 overflow-auto text-text-secondary">
        Move cursor into a highlighted span to see details.
      </div>
    );
  }

  const blocks = (active.rich as DiagnosticRich | null)?.blocks ?? [];

  return (
    <div className="h-full bg-surface border-l border-border p-4 overflow-auto">
      <div className="text-text font-semibold mb-2">{active.code}: {active.message}</div>
      <div className="space-y-4">
        {blocks.map((b, i) => {
          if (b.type === "MarkdownGfm") {
            const html = renderMarkdownToHtml(b.markdown);
            return <div key={i} className="prose prose-invert" dangerouslySetInnerHTML={{ __html: html }} />;
          }
          if (b.type === "Mermaid") {
            // Render as fenced code for now; future: inline SVG render with mermaid.render
            return (
              <pre key={i} className="bg-surface-elevated p-3 rounded border border-border overflow-auto">
{`mermaid\n${b.mermaid}`}
              </pre>
            );
          }
          return null;
        })}
      </div>
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

