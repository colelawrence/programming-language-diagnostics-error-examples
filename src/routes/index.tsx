import { createFileRoute } from "@tanstack/react-router";
import { EditorComponent } from "#src/editor/EditorComponent";
import { createRouter } from "#src/router";
import { wasmAdaptor } from "#src/router/wasmAdaptor";
import { useMemo } from "react";

function Home() {
  const router = useMemo(() => {
    return createRouter({
      adaptor: wasmAdaptor,
    });
  }, []);

  return <EditorComponent router={router} />;
}

export const Route = createFileRoute("/")({
  component: Home,
});
