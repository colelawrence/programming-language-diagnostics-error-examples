# Editor Diagnostics System

## Overview

The editor provides real-time language diagnostics powered by Rust, with a sophisticated error protocol designed for extensibility and span-based error highlighting.

## Error Protocol

### Structure

The diagnostic protocol consists of three main components:

```rust
// 1. Editor Error - The error itself
pub struct EditorError {
    pub code: String,          // e.g., "E001", "W002"
    pub key: String,           // unique within this request
    pub error_kind: ErrorKind, // discriminated union
}

// 2. Error Kind - Specific error type
pub enum ErrorKind {
    SyntaxError { message: String },
    TypeError { expected: String, found: String },
    UndefinedVariable { name: String },
    DuplicateDefinition { name: String },
    InvalidOperation { operation: String, reason: String },
}

// 3. Diagnostic Display - Where to show the error
pub struct DiagnosticDisplay {
    pub error_key: String,     // references EditorError.key
    pub target: SourceCodeSpan, // line/column range
    pub message: String,        // human-readable message
}

// Complete response
pub struct DiagnosticResponse {
    pub errors: Vec<EditorError>,
    pub diagnostics: Vec<DiagnosticDisplay>,
}
```

### Example Flow

```typescript
// 1. User types code
const content = `
let x = undefined;
let String y = 123;
`;

// 2. Send to Rust for analysis
router.analyze_code({ content }).subscribe({
  next: (response: DiagnosticResponse) => {
    // 3. Receive structured errors
    console.log(response.errors);
    // [
    //   {
    //     code: "E001",
    //     key: "err_0",
    //     error_kind: { type: "UndefinedVariable", name: "undefined" }
    //   },
    //   {
    //     code: "E003",
    //     key: "err_1",
    //     error_kind: { type: "TypeError", expected: "String", found: "Number" }
    //   }
    // ]
    
    // 4. Get diagnostic locations
    console.log(response.diagnostics);
    // [
    //   {
    //     error_key: "err_0",
    //     target: { start_line: 2, start_column: 8, end_line: 2, end_column: 17 },
    //     message: "Use of undefined variable"
    //   },
    //   {
    //     error_key: "err_1",
    //     target: { start_line: 3, start_column: 11, end_line: 3, end_column: 16 },
    //     message: "Type mismatch: expected String, found Number"
    //   }
    // ]
  }
});
```

## Features

### Span-Based Highlighting

Errors include precise source code locations:

```typescript
export interface SourceCodeSpan {
  start_line: number;    // 1-based
  start_column: number;  // 0-based
  end_line: number;      // 1-based
  end_column: number;    // 0-based (exclusive)
}
```

Monaco Editor uses these to create decorations:

```typescript
const decorations = diagnostics.map((diag) => ({
  range: new monaco.Range(
    diag.target.start_line,
    diag.target.start_column + 1,
    diag.target.end_line,
    diag.target.end_column + 1,
  ),
  options: {
    className: "text-error underline decoration-wavy",
    hoverMessage: { value: diag.message },
  },
}));
```

### Debounced Analysis

Analysis is debounced to avoid overwhelming the backend:

```typescript
const { diagnostics, isAnalyzing, analyzeCode } = useEditorDiagnostics({
  router,
  debounceMs: 500, // default
});

// Called on every keystroke, but only analyzes after 500ms of inactivity
analyzeCode(content);
```

### Error Categories

Built-in error kinds:

- **SyntaxError** - Parsing errors, malformed syntax
- **TypeError** - Type mismatches, incompatible operations
- **UndefinedVariable** - Use of undeclared variables
- **DuplicateDefinition** - Multiple declarations of same name
- **InvalidOperation** - Illegal operations, warnings

### Extensibility

Adding a new error type is simple:

1. Add to `ErrorKind` enum in `shared-types/src/lib.rs`
2. Implement detection in `editor-core/src/handler.rs`
3. Run codegen - TypeScript types auto-generate
4. UI automatically handles new error type

## Implementation

### Rust Side (`editor-core/src/handler.rs`)

```rust
fn analyze_content(content: &str) -> DiagnosticResponse {
    let mut errors = Vec::new();
    let mut diagnostics = Vec::new();

    for (line_idx, line) in content.lines().enumerate() {
        let line_num = line_idx + 1;

        // Example: detect undefined variables
        if let Some(col) = line.find("undefined") {
            let error_key = format!("err_{}", errors.len());
            errors.push(EditorError {
                code: "E001".to_string(),
                key: error_key.clone(),
                error_kind: ErrorKind::UndefinedVariable {
                    name: "undefined".to_string(),
                },
            });
            diagnostics.push(DiagnosticDisplay {
                error_key,
                target: SourceCodeSpan {
                    start_line: line_num,
                    start_column: col,
                    end_line: line_num,
                    end_column: col + 9,
                },
                message: "Use of undefined variable".to_string(),
            });
        }
    }

    DiagnosticResponse {
        errors,
        diagnostics,
    }
}
```

### TypeScript Side (`src/editor/EditorComponent.tsx`)

```typescript
// 1. Monaco Editor setup
<Editor
  value={content}
  onChange={(value) => setContent(value || "")}
  onMount={(editor) => setEditorInstance(editor)}
/>

// 2. Analyze on change
useEffect(() => {
  analyzeCode(content);
}, [content, analyzeCode]);

// 3. Apply decorations
useEffect(() => {
  if (!editorInstance || !diagnostics) return;

  const decorations = diagnostics.diagnostics.map((diag) => ({
    range: new monaco.Range(
      diag.target.start_line,
      diag.target.start_column + 1,
      diag.target.end_line,
      diag.target.end_column + 1,
    ),
    options: {
      className: "text-error underline decoration-wavy",
      hoverMessage: { value: diag.message },
    },
  }));

  const decorationIds = editorInstance.createDecorationsCollection(decorations);
  return () => decorationIds.clear();
}, [editorInstance, diagnostics]);
```

## UI Components

### Diagnostics Panel

Located at bottom of editor (20-30% height):

```typescript
<DiagnosticsPanel diagnostics={diagnostics} error={error} />
```

Features:
- Lists all errors with codes
- Shows line/column positions
- Groups diagnostics by error
- Styled with terminal theme

### Error Display

Each error shows:
- Error code (e.g., `E001`)
- Error kind message
- Associated diagnostics with locations
- Clickable (future: jump to location)

## Transport Agnostic

Works identically with WASM or WebSocket:

```typescript
// WASM (in-browser analysis)
const router = createRouter({ adaptor: wasmAdaptor });

// WebSocket (server-side analysis)
const router = createRouter({ 
  adaptor: createWebSocketAdaptor({ url: "ws://localhost:10810" })
});

// Same API for both!
router.analyze_code({ content }).subscribe(...);
```

## Performance

- **Debouncing**: 500ms reduces unnecessary analysis
- **Abort Controller**: Cancels pending requests on new input
- **WASM Speed**: 1-5ms analysis time
- **Incremental**: Future optimization - parse only changed sections

## Future Enhancements

### Click to Jump
Click diagnostic → jump to span in editor

### Severity Levels
- Error (red)
- Warning (yellow)
- Info (blue)

### Quick Fixes
```typescript
diagnostics: [{
  target: span,
  message: "Use of undefined",
  fixes: [
    { title: "Declare variable", edits: [...] }
  ]
}]
```

### Multi-file Analysis
```rust
pub struct AnalyzeProjectParams {
    pub files: Vec<FileContent>,
}
```

### Incremental Parsing
Only reparse changed regions for large files.

