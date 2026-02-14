from __future__ import annotations

from dataclasses import dataclass, field
from enum import Enum
from typing import Any

from . import _seonbi


class QuoteOption(str, Enum):
    CurvedQuotes = "curved-quotes"
    VerticalCornerBrackets = "vertical-corner-brackets"
    HorizontalCornerBrackets = "horizontal-corner-brackets"
    Guillemets = "guillemets"
    CurvedSingleQuotesWithQ = "curved-single-quotes-with-q"
    VerticalCornerBracketsWithQ = "vertical-corner-brackets-with-q"
    HorizontalCornerBracketsWithQ = "horizontal-corner-brackets-with-q"


class CiteOption(str, Enum):
    AngleQuotes = "angle-quotes"
    CornerBrackets = "corner-brackets"
    AngleQuotesWithCite = "angle-quotes-with-cite"
    CornerBracketsWithCite = "corner-brackets-with-cite"


class StopOption(str, Enum):
    Horizontal = "horizontal"
    HorizontalWithSlashes = "horizontal-with-slashes"
    Vertical = "vertical"


class HanjaRenderingOption(str, Enum):
    HangulOnly = "hangul-only"
    HanjaInParentheses = "hanja-in-parentheses"
    DisambiguatingHanjaInParentheses = "disambiguating-hanja-in-parentheses"
    HanjaInRuby = "hanja-in-ruby"


@dataclass(slots=True)
class ArrowOption:
    bidir_arrow: bool = False
    double_arrow: bool = False


@dataclass(slots=True)
class HanjaReadingOption:
    initial_sound_law: bool = False
    use_dictionaries: list[str] = field(default_factory=list)
    dictionary: dict[str, str] = field(default_factory=dict)


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


def transform(config: Configuration, input: str) -> str:
    if not isinstance(config, Configuration):
        raise TypeError("config must be a Configuration instance")
    return _seonbi.transform(_configuration_to_dict(config), input)


def ko_kr() -> Configuration:
    return _configuration_from_dict(_seonbi.ko_kr())


def ko_kp() -> Configuration:
    return _configuration_from_dict(_seonbi.ko_kp())


def _configuration_to_dict(config: Configuration) -> dict[str, Any]:
    payload: dict[str, Any] = {
        "content_type": config.content_type,
        "ellipsis": config.ellipsis,
        "em_dash": config.em_dash,
    }

    if config.preset is not None:
        payload["preset"] = config.preset
    if config.quote is not None:
        payload["quote"] = config.quote.value
    if config.cite is not None:
        payload["cite"] = config.cite.value
    if config.stop is not None:
        payload["stop"] = config.stop.value
    if config.arrow is not None:
        payload["arrow"] = {
            "bidir_arrow": config.arrow.bidir_arrow,
            "double_arrow": config.arrow.double_arrow,
        }
    if config.hanja is not None:
        payload["hanja"] = {
            "rendering": config.hanja.rendering.value,
            "reading": {
                "initial_sound_law": config.hanja.reading.initial_sound_law,
                "use_dictionaries": list(config.hanja.reading.use_dictionaries),
                "dictionary": dict(config.hanja.reading.dictionary),
            },
        }

    return payload


def _configuration_from_dict(payload: dict[str, Any]) -> Configuration:
    arrow: ArrowOption | None = None
    arrow_payload = payload.get("arrow")
    if isinstance(arrow_payload, dict):
        arrow = ArrowOption(
            bidir_arrow=bool(arrow_payload.get("bidir_arrow", arrow_payload.get("bidirArrow", False))),
            double_arrow=bool(arrow_payload.get("double_arrow", arrow_payload.get("doubleArrow", False))),
        )

    hanja: HanjaOption | None = None
    hanja_payload = payload.get("hanja")
    if isinstance(hanja_payload, dict):
        reading_payload = hanja_payload.get("reading") if isinstance(hanja_payload.get("reading"), dict) else {}
        initial_sound_law = bool(
            reading_payload.get("initial_sound_law", reading_payload.get("initialSoundLaw", False))
        )
        use_dictionaries = reading_payload.get("use_dictionaries", reading_payload.get("useDictionaries", []))
        dictionary = reading_payload.get("dictionary", {})
        hanja = HanjaOption(
            rendering=HanjaRenderingOption(str(hanja_payload["rendering"])),
            reading=HanjaReadingOption(
                initial_sound_law=initial_sound_law,
                use_dictionaries=list(use_dictionaries) if isinstance(use_dictionaries, list) else [],
                dictionary=dict(dictionary) if isinstance(dictionary, dict) else {},
            ),
        )

    quote = payload.get("quote")
    cite = payload.get("cite")
    stop = payload.get("stop")

    return Configuration(
        content_type=str(payload.get("content_type", payload.get("contentType", "text/html"))),
        preset=payload.get("preset") if isinstance(payload.get("preset"), str) else None,
        quote=QuoteOption(str(quote)) if quote is not None else None,
        cite=CiteOption(str(cite)) if cite is not None else None,
        arrow=arrow,
        ellipsis=bool(payload.get("ellipsis", False)),
        em_dash=bool(payload.get("em_dash", payload.get("emDash", False))),
        stop=StopOption(str(stop)) if stop is not None else None,
        hanja=hanja,
    )


__all__ = [
    "ArrowOption",
    "CiteOption",
    "Configuration",
    "HanjaOption",
    "HanjaReadingOption",
    "HanjaRenderingOption",
    "QuoteOption",
    "StopOption",
    "ko_kp",
    "ko_kr",
    "transform",
]
