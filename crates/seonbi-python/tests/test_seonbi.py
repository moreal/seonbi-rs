import pytest

from seonbi import (
    Configuration,
    HanjaOption,
    HanjaReadingOption,
    HanjaRenderingOption,
    QuoteOption,
    ko_kr,
    transform,
)


def test_ko_kr_returns_configuration() -> None:
    config = ko_kr()
    assert isinstance(config, Configuration)


def test_transform_smoke() -> None:
    output = transform(Configuration(), "<p>\"漢字\"</p>")
    assert isinstance(output, str)


def test_preset_allows_overrides() -> None:
    config = Configuration(preset="ko-kr", quote=QuoteOption.Guillemets)
    output = transform(config, "<p>\"abc\"</p>")
    assert "&#x300a;abc&#x300b;" in output


def test_ko_kr_output_differs_by_content_type() -> None:
    html_config = ko_kr()
    html_output = transform(html_config, "<p>\"abc\"</p>")
    assert "&ldquo;abc&rdquo;" in html_output

    markdown_config = ko_kr()
    markdown_config.content_type = "text/markdown"
    markdown_output = transform(markdown_config, "\"abc\"")
    assert "“abc”" in markdown_output
    assert "&ldquo;" not in markdown_output


@pytest.mark.parametrize(
    ("quote", "content_type", "input_text", "expected"),
    [
        (QuoteOption.CurvedQuotes, "text/html", "<p>\"abc\"</p>", "<p>&ldquo;abc&rdquo;</p>"),
        (
            QuoteOption.CurvedQuotes,
            "application/xhtml+xml",
            "<p>\"abc\"</p>",
            "<p>&ldquo;abc&rdquo;</p>",
        ),
        (QuoteOption.CurvedQuotes, "text/markdown", "\"abc\"", "“abc”\n"),
        (QuoteOption.CurvedQuotes, "text/plain", "\"abc\"", "“abc”"),
        (QuoteOption.Guillemets, "text/html", "<p>\"abc\"</p>", "<p>&#x300a;abc&#x300b;</p>"),
        (
            QuoteOption.Guillemets,
            "application/xhtml+xml",
            "<p>\"abc\"</p>",
            "<p>&#x300a;abc&#x300b;</p>",
        ),
        (QuoteOption.Guillemets, "text/markdown", "\"abc\"", "《abc》\n"),
        (QuoteOption.Guillemets, "text/plain", "\"abc\"", "《abc》"),
    ],
)
def test_quote_content_type_matrix(
    quote: QuoteOption, content_type: str, input_text: str, expected: str
) -> None:
    config = ko_kr()
    config.quote = quote
    config.content_type = content_type
    output = transform(config, input_text)
    assert output == expected


def test_invalid_content_type_raises_value_error() -> None:
    config = Configuration(content_type="invalid/type")
    with pytest.raises(ValueError, match="Invalid content type"):
        transform(config, "<p>text</p>")


def test_invalid_dictionary_id_raises_value_error() -> None:
    config = Configuration(
        hanja=HanjaOption(
            rendering=HanjaRenderingOption.HangulOnly,
            reading=HanjaReadingOption(use_dictionaries=["unknown-dict"]),
        )
    )
    with pytest.raises(ValueError, match="No such dictionary ID"):
        transform(config, "<p>漢字</p>")
