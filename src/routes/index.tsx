import { createFileRoute } from "@tanstack/react-router";
import { PathfinderDemo } from "#src/pathfinder.tsx";

function Home() {
  return <PathfinderDemo />;
}

export const Route = createFileRoute("/")({
  component: Home,
});
