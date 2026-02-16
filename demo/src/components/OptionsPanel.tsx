import { Form, Row, Col, Button } from "react-bootstrap";
import { CustomDictionaryModal } from "./CustomDictionaryModal";
import type {
  AppState,
  Action,
  CustomOptions,
  QuoteOption,
  CiteOption,
  StopOption,
  HanjaRenderingOption,
  ContentType,
} from "../types";

interface OptionsPanelProps {
  state: AppState;
  dispatch: React.Dispatch<Action>;
}

export function OptionsPanel({ state, dispatch }: OptionsPanelProps) {
  const isCustom = state.preset === "custom";
  const opts = state.lastCustomOptions;
  const hanjaActive = isCustom && opts.hanja !== null;
  const arrowActive = isCustom && opts.arrow !== null;

  function updateCustom(partial: Partial<CustomOptions>) {
    dispatch({
      type: "SET_CUSTOM_OPTIONS",
      options: { ...opts, ...partial },
    });
  }

  function setQuote(quote: QuoteOption) {
    updateCustom({ quote });
  }

  function setCite(value: string) {
    const map: Record<string, CiteOption | null> = {
      "": null,
      AngleQuotes: "AngleQuotes",
      AngleQuotesWithCite: "AngleQuotesWithCite",
      CornerBrackets: "CornerBrackets",
      CornerBracketsWithCite: "CornerBracketsWithCite",
    };
    updateCustom({ cite: map[value] ?? null });
  }

  function toggleArrow(checked: boolean) {
    updateCustom({
      arrow: checked ? { bidirArrow: true, doubleArrow: true } : null,
    });
  }

  function toggleBidir(checked: boolean) {
    updateCustom({
      arrow: {
        bidirArrow: checked,
        doubleArrow: opts.arrow?.doubleArrow ?? false,
      },
    });
  }

  function toggleDouble(checked: boolean) {
    updateCustom({
      arrow: {
        bidirArrow: opts.arrow?.bidirArrow ?? false,
        doubleArrow: checked,
      },
    });
  }

  function toggleEllipsis(checked: boolean) {
    updateCustom({ ellipsis: checked });
  }

  function toggleEmDash(checked: boolean) {
    updateCustom({ emDash: checked });
  }

  function setStop(value: string) {
    const map: Record<string, StopOption | null> = {
      "": null,
      Horizontal: "Horizontal",
      HorizontalWithSlashes: "HorizontalWithSlashes",
      Vertical: "Vertical",
    };
    updateCustom({ stop: map[value] ?? null });
  }

  function setHanjaRendering(value: string) {
    const map: Record<string, HanjaRenderingOption | null> = {
      "": null,
      HangulOnly: "HangulOnly",
      HanjaInParentheses: "HanjaInParentheses",
      DisambiguatingHanjaInParentheses: "DisambiguatingHanjaInParentheses",
      HanjaInRuby: "HanjaInRuby",
    };
    const rendering = map[value];
    if (rendering === null) {
      updateCustom({ hanja: null });
    } else {
      updateCustom({
        hanja: {
          rendering,
          reading: opts.hanja?.reading ?? {
            initialSoundLaw: true,
            useDictionaries: new Set<string>(),
            dictionary: {},
          },
        },
      });
    }
  }

  function toggleISL(checked: boolean) {
    if (!opts.hanja) return;
    updateCustom({
      hanja: {
        ...opts.hanja,
        reading: { ...opts.hanja.reading, initialSoundLaw: checked },
      },
    });
  }

  function toggleKrStdict(checked: boolean) {
    if (!opts.hanja) return;
    const dicts = new Set(opts.hanja.reading.useDictionaries);
    if (checked) {
      dicts.add("kr-stdict");
    } else {
      dicts.delete("kr-stdict");
    }
    updateCustom({
      hanja: {
        ...opts.hanja,
        reading: { ...opts.hanja.reading, useDictionaries: dicts },
      },
    });
  }

  function setContentType(value: string) {
    dispatch({ type: "SET_CONTENT_TYPE", contentType: value as ContentType });
  }

  return (
    <Form>
      {/* Row 1: Preset */}
      <Row className="mb-2">
        <Form.Label column sm={1}>
          Preset
        </Form.Label>
        <Col sm={11} className="mt-2">
          <Form.Check
            id="preset-ko-kr"
            inline
            type="radio"
            name="preset"
            label="South Korean"
            checked={state.preset === "ko-kr"}
            onChange={() => dispatch({ type: "SET_PRESET", preset: "ko-kr" })}
          />
          <Form.Check
            id="preset-ko-kp"
            inline
            type="radio"
            name="preset"
            label="North Korean"
            checked={state.preset === "ko-kp"}
            onChange={() => dispatch({ type: "SET_PRESET", preset: "ko-kp" })}
          />
          <Form.Check
            id="preset-custom"
            inline
            type="radio"
            name="preset"
            label="Custom"
            checked={state.preset === "custom"}
            onChange={() => dispatch({ type: "SET_PRESET", preset: "custom" })}
          />
        </Col>
      </Row>

      {/* Row 2: Quotes + Punctuation */}
      <Row className="mb-2">
        <Form.Label column sm={1}>
          Quotes
        </Form.Label>
        <Col sm={6} className="mt-2">
          <Form.Check
            id="quote-curved"
            inline
            type="radio"
            name="quote"
            label="Curved quotes"
            disabled={!isCustom}
            checked={isCustom && opts.quote === "CurvedQuotes"}
            onChange={() => setQuote("CurvedQuotes")}
          />
          <Form.Check
            id="quote-guillemets"
            inline
            type="radio"
            name="quote"
            label="Guillemets"
            disabled={!isCustom}
            checked={isCustom && opts.quote === "Guillemets"}
            onChange={() => setQuote("Guillemets")}
          />
          <Form.Check
            id="quote-curved-q"
            inline
            type="radio"
            name="quote"
            label={<>Curved quotes with &lt;q&gt;</>}
            disabled={!isCustom}
            checked={isCustom && opts.quote === "CurvedSingleQuotesWithQ"}
            onChange={() => setQuote("CurvedSingleQuotesWithQ")}
          />
        </Col>
        <Form.Label column sm={1}>
          Punctuation
        </Form.Label>
        <Col sm={4} className="mt-2">
          <Form.Check
            id="punct-ellipsis"
            inline
            type="checkbox"
            label="Ellipsis"
            disabled={!isCustom}
            checked={isCustom && opts.ellipsis}
            onChange={(e) => toggleEllipsis(e.target.checked)}
          />
          <Form.Check
            id="punct-em-dash"
            inline
            type="checkbox"
            label="Em dash"
            disabled={!isCustom}
            checked={isCustom && opts.emDash}
            onChange={(e) => toggleEmDash(e.target.checked)}
          />
        </Col>
      </Row>

      {/* Row 3: Arrow + Citation */}
      <Row className="mb-2">
        <Form.Label column sm={1}>
          Arrow
        </Form.Label>
        <Col sm={6} className="mt-2">
          <Form.Check
            id="arrow-toggle"
            inline
            type="checkbox"
            label="Arrow"
            disabled={!isCustom}
            checked={arrowActive}
            onChange={(e) => toggleArrow(e.target.checked)}
          />
          <Form.Check
            id="arrow-bidir"
            inline
            type="checkbox"
            label="Bidirection"
            disabled={!arrowActive}
            checked={arrowActive && (opts.arrow?.bidirArrow ?? false)}
            onChange={(e) => toggleBidir(e.target.checked)}
          />
          <Form.Check
            id="arrow-double"
            inline
            type="checkbox"
            label="Double"
            disabled={!arrowActive}
            checked={arrowActive && (opts.arrow?.doubleArrow ?? false)}
            onChange={(e) => toggleDouble(e.target.checked)}
          />
        </Col>
        <Form.Label column sm={1}>
          Citation
        </Form.Label>
        <Col sm={4}>
          <Form.Select
            disabled={!isCustom}
            value={isCustom ? opts.cite ?? "" : ""}
            onChange={(e) => setCite(e.target.value)}
          >
            <option value="">As is</option>
            <option value="AngleQuotes">Angle quotes</option>
            <option value="AngleQuotesWithCite">
              Angle quotes with &lt;cite&gt;
            </option>
            <option value="CornerBrackets">Corner brackets</option>
            <option value="CornerBracketsWithCite">
              Corner brackets with &lt;cite&gt;
            </option>
          </Form.Select>
        </Col>
      </Row>

      {/* Row 4: Hanja */}
      <Row className="mb-2">
        <Form.Label column sm={1}>
          Hanja
        </Form.Label>
        <Col sm={4}>
          <Form.Select
            disabled={!isCustom}
            value={
              isCustom && opts.hanja ? opts.hanja.rendering : ""
            }
            onChange={(e) => setHanjaRendering(e.target.value)}
          >
            <option value="">As is</option>
            <option value="HangulOnly">Hangul only</option>
            <option value="HanjaInParentheses">Hanja in parentheses</option>
            <option value="DisambiguatingHanjaInParentheses">
              Disambiguating hanja in parentheses
            </option>
            <option value="HanjaInRuby">
              Hanja in &lt;ruby&gt;
            </option>
          </Form.Select>
        </Col>
        <Col sm={2}>
          <Button
            variant="outline-secondary"
            size="sm"
            className="p-2"
            disabled={!hanjaActive}
            onClick={() => dispatch({ type: "SHOW_CUSTOM_DICTIONARY" })}
          >
            Custom dictionary ({Object.keys(state.customDictionary).length})
          </Button>
        </Col>
        <Col sm={5} className="mt-2">
          <Form.Check
            id="hanja-isl"
            inline
            type="checkbox"
            label="Initial Sound Law"
            disabled={!hanjaActive}
            checked={hanjaActive && (opts.hanja?.reading.initialSoundLaw ?? false)}
            onChange={(e) => toggleISL(e.target.checked)}
          />
          <Form.Check
            id="hanja-kr-stdict"
            inline
            type="checkbox"
            label="South Korean Standard Dictionary"
            disabled={!hanjaActive}
            checked={
              hanjaActive &&
              (opts.hanja?.reading.useDictionaries.has("kr-stdict") ?? false)
            }
            onChange={(e) => toggleKrStdict(e.target.checked)}
          />
        </Col>
      </Row>

      {/* Row 5: Stop + Type */}
      <Row className="mb-2">
        <Form.Label column sm={1}>
          Stop
        </Form.Label>
        <Col sm={6} className="mt-2">
          <Form.Check
            id="stop-horizontal"
            inline
            type="radio"
            name="stop"
            label="Horizontal"
            disabled={!isCustom}
            checked={isCustom && opts.stop === "Horizontal"}
            onChange={() => setStop("Horizontal")}
          />
          <Form.Check
            id="stop-horizontal-slashes"
            inline
            type="radio"
            name="stop"
            label="Horizontal with slashes"
            disabled={!isCustom}
            checked={isCustom && opts.stop === "HorizontalWithSlashes"}
            onChange={() => setStop("HorizontalWithSlashes")}
          />
          <Form.Check
            id="stop-vertical"
            inline
            type="radio"
            name="stop"
            label="Vertical"
            disabled={!isCustom}
            checked={isCustom && opts.stop === "Vertical"}
            onChange={() => setStop("Vertical")}
          />
          <Form.Check
            id="stop-as-is"
            inline
            type="radio"
            name="stop"
            label="As is"
            disabled={!isCustom}
            checked={isCustom && opts.stop === null}
            onChange={() => setStop("")}
          />
        </Col>
        <Form.Label column sm={1}>
          Type
        </Form.Label>
        <Col sm={4}>
          <Form.Select
            value={state.source.contentType}
            onChange={(e) => setContentType(e.target.value)}
          >
            <option value="text/html">HTML</option>
            <option value="application/xhtml+xml">XHTML</option>
            <option value="text/plain">Plain text</option>
            <option value="text/markdown">Markdown</option>
          </Form.Select>
        </Col>
      </Row>
      <CustomDictionaryModal
        show={state.customDictionaryModalOpen}
        source={state.customDictionarySource}
        entryCount={Object.keys(state.customDictionary).length}
        dispatch={dispatch}
      />
    </Form>
  );
}
