export type ContentType =
  | "text/html"
  | "application/xhtml+xml"
  | "text/plain"
  | "text/markdown";

export type Preset = "ko-kr" | "ko-kp" | "custom";

export type QuoteOption =
  | "CurvedQuotes"
  | "Guillemets"
  | "CurvedSingleQuotesWithQ";

export type CiteOption =
  | "AngleQuotes"
  | "CornerBrackets"
  | "AngleQuotesWithCite"
  | "CornerBracketsWithCite";

export type StopOption = "Horizontal" | "HorizontalWithSlashes" | "Vertical";

export type HanjaRenderingOption =
  | "HangulOnly"
  | "HanjaInParentheses"
  | "DisambiguatingHanjaInParentheses"
  | "HanjaInRuby";

export interface ArrowOption {
  bidirArrow: boolean;
  doubleArrow: boolean;
}

export interface HanjaReadingOption {
  initialSoundLaw: boolean;
  useDictionaries: Set<string>;
  dictionary: Record<string, string>;
}

export interface HanjaOption {
  rendering: HanjaRenderingOption;
  reading: HanjaReadingOption;
}

export interface CustomOptions {
  quote: QuoteOption;
  cite: CiteOption | null;
  arrow: ArrowOption | null;
  ellipsis: boolean;
  emDash: boolean;
  stop: StopOption | null;
  hanja: HanjaOption | null;
}

export interface Source {
  text: string | null;
  html: string;
  contentType: ContentType;
}

export interface TransformResult {
  content: string;
  contentType: ContentType;
}

export interface AppState {
  source: Source;
  preset: Preset;
  lastCustomOptions: CustomOptions;
  lastTransformation: { source: Source; preset: Preset; customOptions: CustomOptions | null } | null;
  result: TransformResult | null;
  error: string | null;
  sourceTab: string;
  resultTab: string;
  customDictionarySource: string;
  customDictionary: Record<string, string>;
  customDictionaryModalOpen: boolean;
}

export type Action =
  | { type: "UPDATE_SOURCE_TEXT"; text: string }
  | { type: "UPDATE_SOURCE_HTML"; html: string }
  | { type: "SET_PRESET"; preset: Preset }
  | { type: "SET_CUSTOM_OPTIONS"; options: CustomOptions }
  | { type: "SET_CONTENT_TYPE"; contentType: ContentType }
  | { type: "SET_SOURCE_TAB"; tab: string }
  | { type: "SET_RESULT_TAB"; tab: string }
  | { type: "TRANSFORM_SUCCESS"; result: TransformResult; source: Source; preset: Preset; customOptions: CustomOptions | null }
  | { type: "TRANSFORM_ERROR"; error: string }
  | { type: "SHOW_CUSTOM_DICTIONARY" }
  | { type: "CLOSE_CUSTOM_DICTIONARY" }
  | { type: "UPDATE_CUSTOM_DICTIONARY_SOURCE"; source: string };
