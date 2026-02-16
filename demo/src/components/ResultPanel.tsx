import { useRef, useEffect } from "react";
import { Tab, Tabs } from "react-bootstrap";
import parse from "html-react-parser";
import hljs from "highlight.js/lib/core";
import xml from "highlight.js/lib/languages/xml";
import type { AppState, Action } from "../types";
import { renderMarkdown } from "../markdown";

hljs.registerLanguage("xml", xml);

interface ResultPanelProps {
  state: AppState;
  dispatch: React.Dispatch<Action>;
}

const TAB_PANE_STYLE: React.CSSProperties = {
  marginTop: "1rem",
  height: "calc(100vh - 450px)",
  width: "540px",
  overflow: "scroll",
};

function RenderView({ state }: { state: AppState }) {
  if (!state.result) return null;

  const { content, contentType } = state.result;

  if (contentType === "text/plain") {
    return (
      <>
        {content.split("\n").map((line, i) => (
          <span key={i}>
            {line}
            <br />
          </span>
        ))}
      </>
    );
  }

  if (contentType === "text/markdown") {
    const html = renderMarkdown(content);
    return <>{parse(html)}</>;
  }

  // HTML or XHTML
  return <>{parse(content)}</>;
}

function CodeView({ state }: { state: AppState }) {
  const codeRef = useRef<HTMLElement>(null);

  useEffect(() => {
    if (
      codeRef.current &&
      state.result &&
      (state.result.contentType === "text/html" ||
        state.result.contentType === "application/xhtml+xml")
    ) {
      codeRef.current.textContent = state.result.content;
      hljs.highlightElement(codeRef.current);
    }
  }, [state.result]);

  if (!state.result) return null;

  const { content, contentType } = state.result;

  if (contentType === "text/html" || contentType === "application/xhtml+xml") {
    return (
      <pre>
        <code ref={codeRef} className="language-xml">
          {content}
        </code>
      </pre>
    );
  }

  return <pre>{content}</pre>;
}

export function ResultPanel({ state, dispatch }: ResultPanelProps) {
  return (
    <Tabs
      activeKey={state.resultTab}
      onSelect={(k) => dispatch({ type: "SET_RESULT_TAB", tab: k ?? "render" })}
    >
      <Tab eventKey="render" title="Render">
        <div style={TAB_PANE_STYLE}>
          <RenderView state={state} />
        </div>
      </Tab>
      <Tab eventKey="code" title="Code">
        <div style={TAB_PANE_STYLE}>
          <CodeView state={state} />
        </div>
      </Tab>
    </Tabs>
  );
}
