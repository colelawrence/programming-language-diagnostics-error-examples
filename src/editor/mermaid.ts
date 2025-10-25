import mermaid from "mermaid";

let initialized = false;

export function initMermaid(isLight: boolean) {
  if (initialized) return;
  mermaid.initialize({
    startOnLoad: false,
    securityLevel: "strict",
    theme: isLight ? "neutral" : "dark",
    fontFamily: "var(--font-mono)",
  });
  initialized = true;
}

const cache = new Map<string, string>();

export async function renderMermaidToSvg(mermaidCode: string): Promise<string> {
  if (cache.has(mermaidCode)) return cache.get(mermaidCode)!;
  const id = `mmd-${Math.random().toString(36).slice(2)}`;
  const { svg } = await mermaid.render(id, mermaidCode);
  cache.set(mermaidCode, svg);
  return svg;
}

export async function renderMermaidToDataUri(mermaidCode: string): Promise<string> {
  const svg = await renderMermaidToSvg(mermaidCode);
  const encoded = encodeURIComponent(svg)
    .replace(/'/g, "%27")
    .replace(/\(/g, "%28")
    .replace(/\)/g, "%29");
  return `data:image/svg+xml;utf8,${encoded}`;
}


