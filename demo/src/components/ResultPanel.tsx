import { useMemo } from "react";
import { Tab, Tabs } from "react-bootstrap";
import parse from "html-react-parser";
import type { HighlighterCore } from "shiki/core";
import type { AppState, Action } from "../types";
import { renderMarkdown } from "../markdown";

interface ResultPanelProps {
  state: AppState;
  dispatch: React.Dispatch<Action>;
  highlighter: HighlighterCore | null;
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

function CodeView({ state, highlighter }: { state: AppState; highlighter: HighlighterCore | null }) {
  const content = state.result?.content ?? null;
  const contentType = state.result?.contentType ?? null;

  const html = useMemo(() => {
    if (
      !content ||
      !(contentType === "text/html" || contentType === "application/xhtml+xml") ||
      !highlighter
    ) {
      return null;
    }
    return highlighter.codeToHtml(content, {
      lang: "xml",
      theme: "github-light",
    });
  }, [content, contentType, highlighter]);

  if (!state.result) return null;
  if (html) return <div dangerouslySetInnerHTML={{ __html: html }} />;
  return <pre>{state.result.content}</pre>;
}

export function ResultPanel({ state, dispatch, highlighter }: ResultPanelProps) {
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
          <CodeView state={state} highlighter={highlighter} />
        </div>
      </Tab>
    </Tabs>
  );
}
