import Editor from "@monaco-editor/react";
import { useEffect, useState } from "react";
import { useTheme } from "#src/useTheme";
import type { Router } from "#src/router/types";
import type { AnalyzerDiagnostics, DiagnosticMessage } from "../../dist-types/index";
import { useEditorDiagnostics } from "./useEditorDiagnostics";
import { ResizablePanel } from "./ResizablePanel";
import type * as Monaco from "monaco-editor";

// on global
declare const monaco: typeof Monaco;


interface EditorComponentProps {
  router: Router;
  initialContent?: string;
}

const INITIAL_CODE = `// Welcome to the editor!
// Try typing 'undefined' to see error detection

let x = undefined;
let String y = 123;

// TODO: implement language features

function example() {
  return { value;
}`;

export function EditorComponent({ router, initialContent = INITIAL_CODE }: EditorComponentProps) {
  const { theme } = useTheme();
  const [content, setContent] = useState(initialContent);
  const [editorInstance, setEditorInstance] = useState<Monaco.editor.IStandaloneCodeEditor | null>(null);
  const { diagnostics, isAnalyzing, error, analyzeCode } = useEditorDiagnostics({ router });

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
      msg.spans.map((span): Monaco.editor.IMarkerData => ({
        startLineNumber: span.start_line,
        startColumn: span.start_column + 1, // Monaco uses 1-based columns
        endLineNumber: span.end_line,
        endColumn: span.end_column + 1,
        message: `${msg.code}: ${msg.message}`,
        severity: getMonacoSeverity(msg.severity),
        source: "Custom Language Analyzer",
      })),
    );

    // Set markers on the model
    monaco.editor.setModelMarkers(model, "custom-analyzer", markers);

    // Create decorations for custom squiggly styling
    // Note: Markers provide the hover behavior, decorations provide custom visual styling
    const decorations = diagnostics.messages.flatMap((msg) =>
      msg.spans.map((span): Monaco.editor.IModelDeltaDecoration => ({
        range: new monaco.Range(
          span.start_line,
          span.start_column + 1,
          span.end_line,
          span.end_column + 1,
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
    // Disable all JavaScript/TypeScript language features BEFORE editor mounts
    monaco.languages.typescript.javascriptDefaults.setDiagnosticsOptions({
      noSemanticValidation: true,
      noSyntaxValidation: true,
      noSuggestionDiagnostics: true,
    });

    monaco.languages.typescript.typescriptDefaults.setDiagnosticsOptions({
      noSemanticValidation: true,
      noSyntaxValidation: true,
      noSuggestionDiagnostics: true,
    });

    // Disable compiler features and standard library
    monaco.languages.typescript.javascriptDefaults.setCompilerOptions({
      noLib: true,
      allowNonTsExtensions: true,
    });

    monaco.languages.typescript.typescriptDefaults.setCompilerOptions({
      noLib: true,
      allowNonTsExtensions: true,
    });

    // Disable eager model sync
    monaco.languages.typescript.javascriptDefaults.setEagerModelSync(false);
    monaco.languages.typescript.typescriptDefaults.setEagerModelSync(false);
  }

  function handleEditorDidMount(editor: Monaco.editor.IStandaloneCodeEditor) {
    setEditorInstance(editor);
  }

  return (
    <div className="h-screen bg-background text-text">
      <ResizablePanel initialTopHeight={500} minTopHeight={200} minBottomHeight={150}>
        {/* Editor */}
        <div className="h-full relative">
          <Editor
            height="100%"
            defaultLanguage="plaintext"
            value={content}
            onChange={(value) => setContent(value || "")}
            beforeMount={handleEditorBeforeMount}
            onMount={handleEditorDidMount}
            theme={theme === "dark" ? "vs-dark" : "vs-light"}
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

        {/* Diagnostics Panel */}
        <DiagnosticsPanel diagnostics={diagnostics} error={error} editorInstance={editorInstance} />
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
    // Set cursor position and selection to the error span
    const selection = new monaco.Selection(
      span.start_line,
      span.start_column + 1, // Monaco uses 1-based columns
      span.end_line,
      span.end_column + 1,
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
                  {msg.spans.map((span, spanIdx) => (
                    <button
                      key={spanIdx}
                      onClick={() => handleDiagnosticClick(span)}
                      className="mt-1 text-text-secondary font-mono text-sm hover:text-primary hover:underline cursor-pointer text-left block"
                    >
                      Line {span.start_line}, Col {span.start_column}
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

