import { marked } from "marked";

export function renderMarkdown(input: string): string {
  return marked.parse(input, { async: false }) as string;
}
