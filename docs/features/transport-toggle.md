# Transport Toggle Feature

## Overview

The frontend now includes a **live transport switcher** that allows you to toggle between WASM (in-browser) and WebSocket (server-side) execution **without reloading the page**.

## Features

✅ **Live Switching** - Toggle transports on the fly  
✅ **Visual Feedback** - Connection status indicator  
✅ **Persistence** - Preference saved to localStorage  
✅ **Graceful Cleanup** - Old router disposed before creating new one  
✅ **Clear UI** - Visual distinction between modes

## UI Components

### Transport Selector
Located in the header of the pathfinder demo:

```
[start: 0] → [end: 5]  |  transport: 🦀 wasm ○ 🌐 ws ●
```

- **🦀 wasm** - In-browser WebAssembly execution
- **🌐 ws** - WebSocket server execution  
- **● Status** - Connection indicator (green = ready, orange = connecting)

### Info Banner
Shows current transport mode with helpful context:

- **Green background** (WASM): "in-browser: pathfinding runs locally via WebAssembly"
- **Cyan background** (WebSocket): "server-side: pathfinding via WebSocket (requires server on :10810)"

## How It Works

### State Management

```typescript
const [transport, setTransport] = useState<Transport>(() => {
  const saved = localStorage.getItem("pathfinder-transport");
  return (saved as Transport) || "wasm";
});
```

### Router Recreation

When transport changes, the component:
1. Saves preference to localStorage
2. Disposes old router
3. Creates new router with selected adaptor
4. Updates connection status

```typescript
useEffect(() => {
  localStorage.setItem("pathfinder-transport", transport);
  
  const newRouter = createRouter({
    adaptor: transport === "wasm"
      ? createWasmAdaptor()
      : createWebSocketAdaptor({ url: "ws://localhost:10810" })
  });
  
  setRouter(newRouter);
  setConnectionStatus("ready");
  
  return () => newRouter.dispose();
}, [transport]);
```

### Path Clearing

When switching transports, the current path is cleared to provide visual feedback:

```typescript
const handleTransportChange = (newTransport: Transport) => {
  setTransport(newTransport);
  setPath([]); // Clear path on switch
};
```

## Usage

### Development

```bash
# Start full environment (includes WebSocket server)
mise run tilt

# Or manually:
mise run server  # Start WebSocket server
mise run dev     # Start Vite dev server
```

### Testing

1. **Start with WASM** (default)
   - Open http://localhost:10880
   - Drag nodes, see pathfinding work instantly
   - All computation in browser

2. **Switch to WebSocket**
   - Click "🌐 ws" radio button
   - Ensure server is running on port 10810
   - Path clears, then recalculates via server
   - Network tab shows WebSocket connection

3. **Switch back to WASM**
   - Click "🦀 wasm" radio button
   - Path clears, then recalculates in browser
   - No network activity

## Architecture Benefits

This demonstrates the core value of the transport-agnostic architecture:

```typescript
// Same router API regardless of transport!
router.findShortestPath(params).subscribe({
  next: (result) => setPath(result.path),
  error: (err) => console.error(err),
});
```

**Application code doesn't know or care** which transport is used. The router handles all the complexity.

## Persistence

The selected transport is saved to `localStorage` with key `pathfinder-transport`:

```javascript
localStorage.getItem("pathfinder-transport")  // "wasm" or "websocket"
```

This means:
- Your preference persists across page reloads
- Each user can have their own preference
- No server-side state needed

## WebSocket Connection

### Auto-Reconnect

The WebSocket adaptor includes auto-reconnect:

```typescript
createWebSocketAdaptor({
  url: "ws://localhost:10810",
  autoReconnect: true,
  reconnectDelay: 1000,
})
```

### Connection States

- **"initializing"** - Creating router
- **"connecting..."** - Establishing WebSocket
- **"ready"** - Connected and ready
- **Connection lost** - Auto-reconnects in 1 second

### Error Handling

If WebSocket server is not running:
- Status shows warning (orange ●)
- Errors displayed in console
- Consider switching back to WASM mode

## Performance Comparison

You can observe the difference:

### WASM (In-Browser)
- **Latency:** ~1-5ms
- **Network:** None
- **CPU:** Client-side
- **Best for:** Interactive UIs, offline use

### WebSocket (Server)
- **Latency:** ~10-50ms (varies by network)
- **Network:** Yes (minimal payload)
- **CPU:** Server-side
- **Best for:** Heavy computation, shared state

## Future Enhancements

### Planned Features

1. **Connection Quality Indicator**
   - Show ping/latency
   - Warn on slow connections

2. **Auto-Fallback**
   - Try WebSocket, fallback to WASM on error
   - Best of both worlds

3. **Transport Stats**
   - Request count
   - Average latency
   - Error rate

4. **Developer Mode**
   - Show request/response payloads
   - Network timing breakdown
   - Performance profiling

### Code Example

```typescript
// Future: add more transports easily
const transports = {
  wasm: createWasmAdaptor(),
  websocket: createWebSocketAdaptor({ url: "ws://localhost:10810" }),
  http: createHttpAdaptor({ url: "http://localhost:3000" }),
  grpc: createGrpcAdaptor({ url: "grpc://localhost:50051" }),
};

const router = createRouter({ 
  adaptor: transports[userPreference] 
});
```

## Troubleshooting

### WebSocket Not Connecting

**Problem:** Status shows "connecting..." or errors in console

**Solutions:**
1. Ensure server is running: `mise run server`
2. Check port 10810 is available
3. Look for server logs
4. Try switching back to WASM

### Preference Not Saving

**Problem:** Transport resets on page reload

**Solutions:**
1. Check browser localStorage is enabled
2. Clear localStorage and try again: `localStorage.clear()`
3. Check browser console for errors

### Path Not Recalculating

**Problem:** Path doesn't update after switching

**Solutions:**
1. Check connection status indicator
2. Try dragging a node to trigger recalculation
3. Check browser console for errors
4. Reload page

## Code Structure

### Files Modified

- **src/pathfinder.tsx** - Added transport toggle and state management
- **src/router/\*.ts** - Already supported multiple transports

### Key Changes

```typescript
// State
const [transport, setTransport] = useState<Transport>("wasm");
const [connectionStatus, setConnectionStatus] = useState<string>("initializing");

// Router creation
useEffect(() => {
  const newRouter = createRouter({
    adaptor: transport === "wasm" 
      ? createWasmAdaptor() 
      : createWebSocketAdaptor({ url: "ws://localhost:10810" })
  });
  setRouter(newRouter);
  return () => newRouter.dispose();
}, [transport]);

// UI
<label>
  <input 
    type="radio" 
    value="wasm"
    checked={transport === "wasm"}
    onChange={(e) => handleTransportChange(e.target.value)}
  />
  🦀 wasm
</label>
```

## Demo

To see the transport toggle in action:

```bash
# 1. Start full environment
mise run tilt

# 2. Open browser
open http://localhost:10880

# 3. Interact
# - Drag nodes to see pathfinding
# - Toggle between WASM and WebSocket
# - Watch network tab when using WebSocket
# - Reload page - preference persists
```

## Conclusion

The transport toggle demonstrates the power of the adapter pattern. The same UI works with completely different backend implementations, switchable at runtime, with zero changes to application logic.

This is the **essence of transport-agnostic architecture**: complete flexibility at the transport layer without touching business logic.
