from dataclasses import dataclass
from enum import Enum


class QuoteOption(str, Enum):
    CurvedQuotes: str
    VerticalCornerBrackets: str
    HorizontalCornerBrackets: str
    Guillemets: str
    CurvedSingleQuotesWithQ: str
    VerticalCornerBracketsWithQ: str
    HorizontalCornerBracketsWithQ: str


class CiteOption(str, Enum):
    AngleQuotes: str
    CornerBrackets: str
    AngleQuotesWithCite: str
    CornerBracketsWithCite: str


class StopOption(str, Enum):
    Horizontal: str
    HorizontalWithSlashes: str
    Vertical: str


class HanjaRenderingOption(str, Enum):
    HangulOnly: str
    HanjaInParentheses: str
    DisambiguatingHanjaInParentheses: str
    HanjaInRuby: str


@dataclass(slots=True)
class ArrowOption:
    bidir_arrow: bool = False
    double_arrow: bool = False


@dataclass(slots=True)
class HanjaReadingOption:
    initial_sound_law: bool = False
    use_dictionaries: list[str] = ...
    dictionary: dict[str, str] = ...


@dataclass(slots=True)
class HanjaOption:
    rendering: HanjaRenderingOption
    reading: HanjaReadingOption


@dataclass(slots=True)
class Configuration:
    content_type: str = "text/html"
    preset: str | None = None
    quote: QuoteOption | None = None
    cite: CiteOption | None = None
    arrow: ArrowOption | None = None
    ellipsis: bool = False
    em_dash: bool = False
    stop: StopOption | None = None
    hanja: HanjaOption | None = None


def transform(config: Configuration, input: str) -> str: ...
def ko_kr() -> Configuration: ...
def ko_kp() -> Configuration: ...
