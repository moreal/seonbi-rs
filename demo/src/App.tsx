import { useReducer, useCallback, useEffect } from "react";
import { Container, Row, Col } from "react-bootstrap";
import type { Configuration } from "@seonbi/wasm";
import { transform } from "./wasm";
import { useWasm } from "./hooks/useWasm";
import { useShiki } from "./hooks/useShiki";
import { INITIAL_CONTENT, DEFAULT_CUSTOM_OPTIONS } from "./constants";
import type {
  AppState,
  Action,
  CustomOptions,
} from "./types";
import { renderMarkdown } from "./markdown";
import { SourcePanel } from "./components/SourcePanel";
import { ResultPanel } from "./components/ResultPanel";
import { OptionsPanel } from "./components/OptionsPanel";
import { TransformButton } from "./components/TransformButton";
import { GitHubCorner } from "./components/GitHubCorner";

function createInitialState(): AppState {
  return {
    source: {
      text: INITIAL_CONTENT,
      html: renderMarkdown(INITIAL_CONTENT),
      contentType: "text/html",
    },
    preset: "ko-kr",
    lastCustomOptions: { ...DEFAULT_CUSTOM_OPTIONS },
    lastTransformation: null,
    result: null,
    error: null,
    sourceTab: "commonmark",
    resultTab: "render",
    customDictionarySource: "",
    customDictionary: {},
    customDictionaryModalOpen: false,
  };
}

function reducer(state: AppState, action: Action): AppState {
  switch (action.type) {
    case "UPDATE_SOURCE_TEXT": {
      return {
        ...state,
        source: {
          ...state.source,
          text: action.text,
          html: renderMarkdown(action.text),
        },
      };
    }
    case "UPDATE_SOURCE_HTML": {
      return {
        ...state,
        source: {
          ...state.source,
          text: null,
          html: action.html,
        },
      };
    }
    case "SET_PRESET": {
      return {
        ...state,
        preset: action.preset,
        lastCustomOptions:
          action.preset === "custom"
            ? state.lastCustomOptions
            : state.lastCustomOptions,
      };
    }
    case "SET_CUSTOM_OPTIONS": {
      return {
        ...state,
        preset: "custom",
        lastCustomOptions: action.options,
      };
    }
    case "SET_CONTENT_TYPE": {
      const isCurrentHtml =
        state.source.contentType === "text/html" ||
        state.source.contentType === "application/xhtml+xml";
      const isNewHtml =
        action.contentType === "text/html" ||
        action.contentType === "application/xhtml+xml";
      return {
        ...state,
        source: {
          ...state.source,
          contentType: action.contentType,
        },
        sourceTab:
          isCurrentHtml && !isNewHtml ? "commonmark" : state.sourceTab,
      };
    }
    case "SET_SOURCE_TAB": {
      return { ...state, sourceTab: action.tab };
    }
    case "SET_RESULT_TAB": {
      return { ...state, resultTab: action.tab };
    }
    case "TRANSFORM_SUCCESS": {
      return {
        ...state,
        result: action.result,
        error: null,
        lastTransformation: {
          source: action.source,
          preset: action.preset,
          customOptions: action.customOptions,
          customDictionary: action.customDictionary,
        },
      };
    }
    case "TRANSFORM_ERROR": {
      return {
        ...state,
        error: action.error,
      };
    }
    case "SHOW_CUSTOM_DICTIONARY": {
      const source = Object.entries(state.customDictionary)
        .map(([k, v]) => `${k} \u2192 ${v}\n`)
        .join("");
      return {
        ...state,
        customDictionaryModalOpen: true,
        customDictionarySource: source,
      };
    }
    case "CLOSE_CUSTOM_DICTIONARY": {
      return { ...state, customDictionaryModalOpen: false };
    }
    case "UPDATE_CUSTOM_DICTIONARY_SOURCE": {
      const arrowPattern = / *-> */g;
      const incompletePattern = /(^|(?:.|\n)*\n)((?:[^-\u2192\n]|-[^>\n])*)\n+$/;
      const completePattern = /(?:^|\n)((?:[^-\u2192\n]|-[^>])+) *(?:->|\u2192) ([^\n]+)/g;

      let newSource = action.source.replace(arrowPattern, " \u2192 ");
      const incompleteMatch = incompletePattern.exec(newSource);
      if (incompleteMatch) {
        const prefix = incompleteMatch[1] ?? "";
        const incomplete = incompleteMatch[2] ?? "";
        newSource = prefix + incomplete + " \u2192 ";
      }

      const dict: Record<string, string> = {};
      for (const match of newSource.matchAll(completePattern)) {
        const key = match[1]?.trim();
        const value = match[2]?.trim();
        if (key && value) {
          dict[key] = value;
        }
      }

      return {
        ...state,
        customDictionarySource: newSource,
        customDictionary: dict,
      };
    }
  }
}

export function buildConfiguration(
  state: AppState
): Configuration {
  const contentType = state.source.contentType;

  if (state.preset === "ko-kr" || state.preset === "ko-kp") {
    return {
      contentType,
      preset: state.preset,
      quote: undefined,
      cite: undefined,
      arrow: undefined,
      ellipsis: undefined,
      emDash: undefined,
      stop: undefined,
      hanja: undefined,
    };
  }

  const opts = state.lastCustomOptions;
  return {
    contentType,
    preset: undefined,
    quote: opts.quote,
    cite: opts.cite ?? undefined,
    arrow: opts.arrow ?? undefined,
    ellipsis: opts.ellipsis,
    emDash: opts.emDash,
    stop: opts.stop ?? undefined,
    hanja: opts.hanja
      ? {
          rendering: opts.hanja.rendering,
          reading: {
            initialSoundLaw: opts.hanja.reading.initialSoundLaw,
            useDictionaries: Array.from(opts.hanja.reading.useDictionaries),
            dictionary: new Map(Object.entries(state.customDictionary)),
          },
        }
      : undefined,
  };
}

function getCustomOptionsForComparison(state: AppState): CustomOptions | null {
  return state.preset === "custom" ? state.lastCustomOptions : null;
}

export function buildHttpRequestBody(state: AppState): object {
  const contentType = state.source.contentType;
  const isHtml =
    contentType === "text/html" || contentType === "application/xhtml+xml";
  const content = isHtml
    ? state.source.html
    : state.source.text ?? "";

  if (state.preset !== "custom") {
    return {
      content,
      contentType,
      preset: state.preset,
    };
  }

  const opts = state.lastCustomOptions;
  const body: Record<string, unknown> = {
    content,
    contentType,
    quote: opts.quote,
    cite: opts.cite,
    ellipsis: opts.ellipsis,
    emDash: opts.emDash,
    stop: opts.stop,
  };

  if (opts.arrow) {
    body.arrow = {
      bidir: opts.arrow.bidirArrow,
      double: opts.arrow.doubleArrow,
    };
  } else {
    body.arrow = null;
  }

  if (opts.hanja) {
    body.hanja = {
      rendering: opts.hanja.rendering,
      reading: {
        initialSoundLaw: opts.hanja.reading.initialSoundLaw,
        useDictionaries: Array.from(opts.hanja.reading.useDictionaries),
        dictionary: state.customDictionary,
      },
    };
  } else {
    body.hanja = null;
  }

  return body;
}

export function buildWasmExample(state: AppState): string {
  const config = buildExampleConfig(state, "wasm");
  return `import init, { transform } from "@seonbi/wasm";

await init();
const result = transform(${config}, input);`;
}

export function buildNodeExample(state: AppState): string {
  const config = buildExampleConfig(state, "node");
  return `import { transform } from "@seonbi/node";

const result = transform(${config}, input);`;
}

export function buildPythonExample(state: AppState): string {
  if (state.preset === "ko-kr") {
    return `from seonbi import ko_kr, transform

config = ko_kr()
config.content_type = ${JSON.stringify(state.source.contentType)}
result = transform(config, input)`;
  }
  if (state.preset === "ko-kp") {
    return `from seonbi import ko_kp, transform

config = ko_kp()
config.content_type = ${JSON.stringify(state.source.contentType)}
result = transform(config, input)`;
  }

  const opts = state.lastCustomOptions;
  const lines = [
    "from seonbi import (",
    "    ArrowOption,",
    "    CiteOption,",
    "    Configuration,",
    "    HanjaOption,",
    "    HanjaReadingOption,",
    "    HanjaRenderingOption,",
    "    QuoteOption,",
    "    StopOption,",
    "    transform,",
    ")",
    "",
  ];

  const configArgs: string[] = [
    `    content_type=${JSON.stringify(state.source.contentType)},`,
    `    quote=QuoteOption.${opts.quote},`,
  ];

  if (opts.cite) {
    configArgs.push(`    cite=CiteOption.${opts.cite},`);
  }

  if (opts.arrow) {
    configArgs.push(
      `    arrow=ArrowOption(bidir_arrow=${opts.arrow.bidirArrow ? "True" : "False"}, double_arrow=${opts.arrow.doubleArrow ? "True" : "False"}),`
    );
  }

  configArgs.push(`    ellipsis=${opts.ellipsis ? "True" : "False"},`);
  configArgs.push(`    em_dash=${opts.emDash ? "True" : "False"},`);

  if (opts.stop) {
    configArgs.push(`    stop=StopOption.${opts.stop},`);
  }

  if (opts.hanja) {
    const dicts = Array.from(opts.hanja.reading.useDictionaries)
      .map((d) => JSON.stringify(d))
      .join(", ");
    const dictEntries = Object.entries(state.customDictionary);
    const dictArg =
      dictEntries.length > 0
        ? `\n            dictionary={${dictEntries.map(([k, v]) => `${JSON.stringify(k)}: ${JSON.stringify(v)}`).join(", ")}},`
        : "";
    configArgs.push(
      `    hanja=HanjaOption(
        rendering=HanjaRenderingOption.${opts.hanja.rendering},
        reading=HanjaReadingOption(
            initial_sound_law=${opts.hanja.reading.initialSoundLaw ? "True" : "False"},
            use_dictionaries=[${dicts}],${dictArg}
        ),
    ),`
    );
  }

  lines.push(`config = Configuration(`);
  lines.push(...configArgs);
  lines.push(`)`);
  lines.push(`result = transform(config, input)`);
  return lines.join("\n");
}

function buildExampleConfig(
  state: AppState,
  _target: "wasm" | "node"
): string {
  if (state.preset !== "custom") {
    return JSON.stringify(
      {
        contentType: state.source.contentType,
        preset: state.preset,
      },
      null,
      2
    );
  }

  const opts = state.lastCustomOptions;
  const config: Record<string, unknown> = {
    contentType: state.source.contentType,
    quote: opts.quote,
  };

  if (opts.cite) config.cite = opts.cite;
  if (opts.arrow) {
    config.arrow = {
      bidirArrow: opts.arrow.bidirArrow,
      doubleArrow: opts.arrow.doubleArrow,
    };
  }
  config.ellipsis = opts.ellipsis;
  config.emDash = opts.emDash;
  if (opts.stop) config.stop = opts.stop;
  if (opts.hanja) {
    const reading: Record<string, unknown> = {
      initialSoundLaw: opts.hanja.reading.initialSoundLaw,
      useDictionaries: Array.from(opts.hanja.reading.useDictionaries),
    };
    if (Object.keys(state.customDictionary).length > 0) {
      reading.dictionary = state.customDictionary;
    }
    config.hanja = {
      rendering: opts.hanja.rendering,
      reading,
    };
  }

  return JSON.stringify(config, null, 2);
}

function isSourceUnchanged(state: AppState): boolean {
  if (!state.lastTransformation) return false;
  const last = state.lastTransformation;
  return (
    last.source.text === state.source.text &&
    last.source.html === state.source.html &&
    last.source.contentType === state.source.contentType &&
    last.preset === state.preset &&
    JSON.stringify(last.customOptions) ===
      JSON.stringify(getCustomOptionsForComparison(state)) &&
    (state.preset !== "custom" ||
      !state.lastCustomOptions.hanja ||
      JSON.stringify(last.customDictionary) ===
        JSON.stringify(state.customDictionary))
  );
}

export default function App() {
  const [state, dispatch] = useReducer(reducer, null, createInitialState);
  const { ready: wasmReady, error: wasmError } = useWasm();
  const { highlighter } = useShiki();

  const handleTransform = useCallback(() => {
    try {
      const config = buildConfiguration(state);
      const isHtml =
        state.source.contentType === "text/html" ||
        state.source.contentType === "application/xhtml+xml";
      const input = isHtml
        ? state.source.html
        : state.source.text ?? "";
      const output = transform(config, input);
      dispatch({
        type: "TRANSFORM_SUCCESS",
        result: { content: output, contentType: state.source.contentType },
        source: { ...state.source },
        preset: state.preset,
        customOptions: getCustomOptionsForComparison(state),
        customDictionary: { ...state.customDictionary },
      });
    } catch (err) {
      dispatch({ type: "TRANSFORM_ERROR", error: String(err) });
    }
  }, [state]);

  useEffect(() => {
    if (wasmReady) {
      handleTransform();
    }
  }, [wasmReady]);

  const sourceUnchanged = isSourceUnchanged(state);

  return (
    <>
      <GitHubCorner />
      <Container className="mt-3 safe-area-bottom">
        <Row>
          <Col>
            <SourcePanel state={state} dispatch={dispatch} highlighter={highlighter} />
          </Col>
          <Col>
            <ResultPanel state={state} dispatch={dispatch} highlighter={highlighter} />
          </Col>
        </Row>
        <Row>
          <Col>
            <OptionsPanel state={state} dispatch={dispatch} />
          </Col>
        </Row>
        <Row className="mt-3">
          <Col>
            <TransformButton
              wasmReady={wasmReady}
              wasmError={wasmError}
              sourceUnchanged={sourceUnchanged}
              error={state.error}
              onTransform={handleTransform}
            />
          </Col>
        </Row>
      </Container>
    </>
  );
}
