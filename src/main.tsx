import { RouterProvider, createRouter } from "@tanstack/react-router";
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { routeTree } from "./routeTree.gen.ts";
import "./styles.css";

const router = createRouter({
  routeTree,
  // Vite's BASE_URL matches the `base` config option (https://vitejs.dev/guide/env-and-mode.html#env-variables)
  // This ensures TanStack Router uses the correct base path for GitHub Pages deployment
  basepath: import.meta.env.BASE_URL as string,
});

declare module "@tanstack/react-router" {
  interface Register {
    router: typeof router;
  }
}

const rootElement = document.getElementById("root");
if (!rootElement) throw new Error("Root element not found");

createRoot(rootElement).render(
  <StrictMode>
    <RouterProvider router={router} />
  </StrictMode>,
);
