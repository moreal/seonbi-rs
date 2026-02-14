from seonbi import Configuration, ko_kr, transform


def test_ko_kr_returns_configuration() -> None:
    config = ko_kr()
    assert isinstance(config, Configuration)


def test_transform_smoke() -> None:
    output = transform(Configuration(), "<p>\"漢字\"</p>")
    assert isinstance(output, str)
