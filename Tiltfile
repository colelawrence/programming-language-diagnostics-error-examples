# Tiltfile for WASM Pathfinder Development

# Local resource for building WASM module with watch
local_resource(
    'wasm-build',
    cmd='mise run --force wasm:dev',
    labels=['build'],
    allow_parallel=False,
    auto_init=False
)

# Local resource for WebSocket server
local_resource(
    'websocket-server',
    serve_cmd='mise run server',
    labels=['serve'],
    readiness_probe=probe(
        tcp_socket=tcp_socket_action(port=10810)
    ),
    links=[
        link('http://localhost:10810', 'WebSocket Server')
    ]
)

# Local resource for Vite dev server
# Depends on wasm-build to ensure WASM is ready before starting
local_resource(
    'vite-server',
    serve_cmd='mise run dev',
    labels=['serve'],
    readiness_probe=probe(
        http_get=http_get_action(port=10880, path='/')
    ),
    links=[
        link('http://localhost:10880', 'Dev Server')
    ]
)

# Optional: Type checking as a separate resource
local_resource(
    'typecheck',
    serve_cmd='mise watch -t typecheck',
    labels=['lint'],
    auto_init=False
)

# Manual test task
local_resource(
    'tests',
    cmd='vitest run',
    labels=['test'],
    trigger_mode=TRIGGER_MODE_MANUAL,
    auto_init=False
)
