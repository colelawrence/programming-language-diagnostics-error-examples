# Adding New Error Types

This guide shows how to add a new error type to the language diagnostics system.

## Overview

The error system is fully extensible. Adding a new error type requires:
1. Updating the Rust enum
2. Implementing detection logic
3. Running codegen (TypeScript types auto-generate)
4. UI automatically handles the new type

## Step-by-Step Example

Let's add a `CircularDependency` error type.

### 1. Define Error in `shared-types/src/lib.rs`

```rust
#[protocol("wasm")]
pub enum ErrorKind {
    // Existing errors...
    SyntaxError { message: String },
    TypeError { expected: String, found: String },
    UndefinedVariable { name: String },
    DuplicateDefinition { name: String },
    InvalidOperation { operation: String, reason: String },
    
    // ← Add your new error here
    CircularDependency { 
        module_name: String,
        dependency_chain: Vec<String>,
    },
}
```

### 2. Implement Detection in `editor-core/src/handler.rs`

```rust
fn analyze_content(content: &str) -> DiagnosticResponse {
    let mut errors = Vec::new();
    let mut diagnostics = Vec::new();

    // Existing detection logic...

    // Add your detection logic
    if let Some(circular_dep) = detect_circular_dependency(content) {
        let error_key = format!("err_{}", errors.len());
        errors.push(EditorError {
            code: "E005".to_string(),
            key: error_key.clone(),
            error_kind: ErrorKind::CircularDependency {
                module_name: circular_dep.module,
                dependency_chain: circular_dep.chain,
            },
        });
        diagnostics.push(DiagnosticDisplay {
            error_key,
            target: circular_dep.span,
            message: format!(
                "Circular dependency detected: {}",
                circular_dep.chain.join(" → ")
            ),
        });
    }

    DiagnosticResponse { errors, diagnostics }
}

fn detect_circular_dependency(content: &str) -> Option<CircularDep> {
    // Your detection logic here
    None // placeholder
}
```

### 3. Run Codegen

```bash
cd shared-types
cargo test --features codegen generate_typescript -- --ignored
```

This generates TypeScript types in `dist-types/index.ts`:

```typescript
export type ErrorKind =
  | { type: "SyntaxError"; message: string; }
  | { type: "TypeError"; expected: string; found: string; }
  | { type: "UndefinedVariable"; name: string; }
  | { type: "DuplicateDefinition"; name: string; }
  | { type: "InvalidOperation"; operation: string; reason: string; }
  | { type: "CircularDependency"; module_name: string; dependency_chain: string[]; };
  //   ↑ Auto-generated!
```

### 4. Use in TypeScript

The UI automatically handles the new error type! But you can add custom handling:

```typescript
function getErrorKindMessage(errorKind: ErrorKind): string {
  if (errorKind.type === 'CircularDependency') {
    return `Circular Dependency: ${errorKind.module_name} (${errorKind.dependency_chain.join(' → ')})`;
  }
  // ... handle other types
}
```

## Error Code Conventions

- `E001-E099`: Syntax errors
- `E100-E199`: Type errors
- `E200-E299`: Name resolution errors
- `E300-E399`: Control flow errors
- `W001-W099`: Warnings
- `I001-I099`: Info messages

## Testing Your Error

### Rust Test

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circular_dependency() {
        let content = r#"
            import A from './A';
            // A imports B
            // B imports this file
        "#;
        
        let result = analyze_content(content);
        assert_eq!(result.errors.len(), 1);
        assert_eq!(result.errors[0].code, "E005");
        
        if let ErrorKind::CircularDependency { module_name, .. } = &result.errors[0].error_kind {
            assert_eq!(module_name, "A");
        } else {
            panic!("Expected CircularDependency error");
        }
    }
}
```

### Integration Test

```typescript
import { describe, it, expect } from 'vitest';
import { createRouter } from '#src/router';
import { wasmAdaptor } from '#src/router/wasmAdaptor';

describe('Circular Dependency Detection', () => {
  it('should detect circular dependencies', async () => {
    const router = createRouter({ adaptor: wasmAdaptor });
    
    const result = await router.analyze_code({
      content: 'import A from "./A"; // circular',
      file_path: null,
    }).first();
    
    expect(result.errors).toHaveLength(1);
    expect(result.errors[0].code).toBe('E005');
    expect(result.errors[0].error_kind.type).toBe('CircularDependency');
  });
});
```

## Complex Errors with Multiple Diagnostics

One error can have multiple diagnostic locations:

```rust
// Error with multiple locations
let error_key = format!("err_{}", errors.len());
errors.push(EditorError {
    code: "E005".to_string(),
    key: error_key.clone(),
    error_kind: ErrorKind::CircularDependency { /* ... */ },
});

// Multiple diagnostics for same error
for location in circular_chain_locations {
    diagnostics.push(DiagnosticDisplay {
        error_key: error_key.clone(),
        target: location.span,
        message: format!("Part of circular dependency: {}", location.name),
    });
}
```

UI will group these diagnostics under one error.

## Tips

1. **Make errors informative**: Include all relevant context in the error kind
2. **Precise spans**: Provide accurate line/column information
3. **Clear messages**: Write user-friendly diagnostic messages
4. **Test thoroughly**: Add tests for edge cases
5. **Document error codes**: Add to error code registry

## Error Severity (Future)

When implementing severity levels:

```rust
pub enum Severity {
    Error,
    Warning,
    Info,
    Hint,
}

pub struct EditorError {
    pub code: String,
    pub key: String,
    pub severity: Severity,  // ← Add this
    pub error_kind: ErrorKind,
}
```

## Related Files

- `shared-types/src/lib.rs` - Error type definitions
- `editor-core/src/handler.rs` - Detection logic
- `src/editor/EditorComponent.tsx` - Error display
- `dist-types/index.ts` - Generated TypeScript types (auto-generated)

