import { Outlet, createRootRoute } from "@tanstack/react-router";
import { useTheme } from "#src/useTheme.ts";

function RootLayout() {
  const { theme, toggleTheme } = useTheme();

  return (
    <div className="min-h-screen bg-background text-text">
      <header className="flex items-center justify-between border-b border-border p-4">
        <h1 className="text-primary">$ pathfinder</h1>
        <button
          type="button"
          onClick={toggleTheme}
          className="px-3 py-1 border border-border bg-surface hover:bg-surface-elevated hover:border-border-focus transition-colors"
          aria-label="Toggle theme"
        >
          {theme === "dark" ? "☀" : "☾"}
        </button>
      </header>
      <main className="p-6">
        <Outlet />
      </main>
    </div>
  );
}

export const Route = createRootRoute({
  component: RootLayout,
});
