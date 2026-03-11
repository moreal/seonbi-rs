import { useEffect, useMemo, useRef, useState } from "react";
import type { HighlighterCore } from "shiki/core";

type HighlightLanguage = "markdown" | "xml";

interface HighlightedEditorProps {
  value: string;
  onChange: (value: string) => void;
  language: HighlightLanguage | null;
  highlighter: HighlighterCore | null;
}

function syncScrollPosition(
  textarea: HTMLTextAreaElement | null,
  highlight: HTMLDivElement | null,
): void {
  if (!textarea || !highlight) return;
  highlight.scrollTop = textarea.scrollTop;
  highlight.scrollLeft = textarea.scrollLeft;
}

export function HighlightedEditor({
  value,
  onChange,
  language,
  highlighter,
}: HighlightedEditorProps) {
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const highlightRef = useRef<HTMLDivElement>(null);
  const [isComposing, setIsComposing] = useState(false);

  const html = useMemo(() => {
    if (!highlighter || !language) {
      return null;
    }

    return highlighter.codeToHtml(value || " ", {
      lang: language,
      theme: "github-light",
    });
  }, [value, language, highlighter]);

  const showHighlight = html !== null && !isComposing;

  useEffect(() => {
    syncScrollPosition(textareaRef.current, highlightRef.current);
  }, [html, showHighlight]);

  return (
    <div className="highlighted-editor">
      {showHighlight && (
        <div
          ref={highlightRef}
          aria-hidden="true"
          className="highlighted-editor__highlight"
          dangerouslySetInnerHTML={{ __html: html }}
        />
      )}
      <textarea
        ref={textareaRef}
        className={`highlighted-editor__input${showHighlight ? " highlighted-editor__input--overlay" : ""}`}
        spellCheck={false}
        value={value}
        wrap="off"
        onChange={(event) => onChange(event.target.value)}
        onCompositionEnd={() => setIsComposing(false)}
        onCompositionStart={() => setIsComposing(true)}
        onScroll={() => syncScrollPosition(textareaRef.current, highlightRef.current)}
      />
    </div>
  );
}
