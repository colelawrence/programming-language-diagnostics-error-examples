import { marked } from "marked";
import DOMPurify from "dompurify";

export const collapseMarkdown = (markdown: string, maxLines = 10): string => {
  const lines = markdown.split("\n");
  return lines.length > maxLines ? lines.slice(0, maxLines).join("\n") + "\n…" : markdown;
};

export const renderMarkdownToHtml = (markdown: string): string => {
  const raw = marked.parse(markdown, { async: false }) as string;
  return DOMPurify.sanitize(raw, { USE_PROFILES: { html: true } });
};


