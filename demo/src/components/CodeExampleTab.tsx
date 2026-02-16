import { useMemo } from "react";
import type { HighlighterCore } from "shiki/core";

interface CodeExampleTabProps {
  code: string;
  language: "javascript" | "python" | "http";
  highlighter: HighlighterCore | null;
}

export function CodeExampleTab({ code, language, highlighter }: CodeExampleTabProps) {
  const html = useMemo(() => {
    if (language === "http" || !highlighter) return null;
    return highlighter.codeToHtml(code, {
      lang: language,
      theme: "github-light",
    });
  }, [code, language, highlighter]);

  if (!html) {
    return <pre>{code}</pre>;
  }

  return <div dangerouslySetInnerHTML={{ __html: html }} />;
}
