import { Tab, Tabs } from "react-bootstrap";
import type { HighlighterCore } from "shiki/core";
import type { AppState, Action } from "../types";
import {
  buildHttpRequestBody,
  buildWasmExample,
  buildNodeExample,
  buildPythonExample,
} from "../App";
import { CodeExampleTab } from "./CodeExampleTab";
import { HighlightedEditor } from "./HighlightedEditor";

interface SourcePanelProps {
  state: AppState;
  dispatch: React.Dispatch<Action>;
  highlighter: HighlighterCore | null;
}

const CODE_TAB_PANE_STYLE: React.CSSProperties = {
  marginTop: "1rem",
  height: "calc(100vh - 450px)",
  width: "540px",
  overflow: "scroll",
};

const EDITOR_TAB_PANE_STYLE: React.CSSProperties = {
  ...CODE_TAB_PANE_STYLE,
  overflow: "hidden",
};

function buildHttpRequestText(state: AppState): string {
  const body = buildHttpRequestBody(state);
  const json = JSON.stringify(body, null, 2);
  return `POST / HTTP/1.1\nHost: localhost:3800\nContent-Type: application/json\n\n${json}`;
}

export function SourcePanel({ state, dispatch, highlighter }: SourcePanelProps) {
  const isHtml =
    state.source.contentType === "text/html" ||
    state.source.contentType === "application/xhtml+xml";

  const markdownLabel = state.source.contentType === "text/plain" ? "Text" : "Markdown";

  const textValue =
    state.source.text ??
    "(HTML cannot be reverted to Markdown.  If you change this Markdown text, the raw HTML you wrote will go.)";

  return (
    <Tabs
      activeKey={state.sourceTab}
      onSelect={(k) => dispatch({ type: "SET_SOURCE_TAB", tab: k ?? "commonmark" })}
    >
      <Tab eventKey="commonmark" title={markdownLabel}>
        <div style={EDITOR_TAB_PANE_STYLE}>
          <HighlightedEditor
            value={textValue}
            language={state.source.contentType === "text/plain" ? null : "markdown"}
            highlighter={highlighter}
            onChange={(text) => dispatch({ type: "UPDATE_SOURCE_TEXT", text })}
          />
        </div>
      </Tab>
      {isHtml && (
        <Tab eventKey="html" title="HTML">
          <div style={EDITOR_TAB_PANE_STYLE}>
            <HighlightedEditor
              value={state.source.html}
              language="xml"
              highlighter={highlighter}
              onChange={(html) => dispatch({ type: "UPDATE_SOURCE_HTML", html })}
            />
          </div>
        </Tab>
      )}
      <Tab eventKey="http" title="HTTP">
        <div style={CODE_TAB_PANE_STYLE}>
          <CodeExampleTab
            code={buildHttpRequestText(state)}
            language="http"
            highlighter={highlighter}
          />
        </div>
      </Tab>
      <Tab eventKey="wasm" title="WASM">
        <div style={CODE_TAB_PANE_STYLE}>
          <CodeExampleTab
            code={buildWasmExample(state)}
            language="javascript"
            highlighter={highlighter}
          />
        </div>
      </Tab>
      <Tab eventKey="nodejs" title="Node.js">
        <div style={CODE_TAB_PANE_STYLE}>
          <CodeExampleTab
            code={buildNodeExample(state)}
            language="javascript"
            highlighter={highlighter}
          />
        </div>
      </Tab>
      <Tab eventKey="python" title="Python">
        <div style={CODE_TAB_PANE_STYLE}>
          <CodeExampleTab
            code={buildPythonExample(state)}
            language="python"
            highlighter={highlighter}
          />
        </div>
      </Tab>
    </Tabs>
  );
}
