use std::collections::BTreeSet;

use seonbi::{
    k_hangul_data, k_hangul_data_result, CharacterSet, HanjaReadingCitation, Purpose,
};

#[test]
fn k_hangul_data_is_loaded() {
    assert!(k_hangul_data_result().is_ok());
}

#[test]
fn k_hangul_data_contains_readings() {
    let readings = k_hangul_data().get(&'識').expect("must exist");
    assert_eq!(
        readings.get(&'식'),
        Some(&HanjaReadingCitation(
            CharacterSet::KS_X_1001,
            BTreeSet::from([Purpose::Education])
        ))
    );
    assert_eq!(
        readings.get(&'지'),
        Some(&HanjaReadingCitation(
            CharacterSet::KS_X_1001,
            BTreeSet::from([Purpose::PersonalName])
        ))
    );
}

#[test]
fn hanja_reading_citation_parses_json_text() {
    let cases = [
        (
            "\"\"",
            Some(HanjaReadingCitation(CharacterSet::NonStandard, BTreeSet::new())),
        ),
        (
            "\"E\"",
            Some(HanjaReadingCitation(
                CharacterSet::NonStandard,
                BTreeSet::from([Purpose::Education]),
            )),
        ),
        (
            "\"N\"",
            Some(HanjaReadingCitation(
                CharacterSet::NonStandard,
                BTreeSet::from([Purpose::PersonalName]),
            )),
        ),
        (
            "\"EN\"",
            Some(HanjaReadingCitation(
                CharacterSet::NonStandard,
                BTreeSet::from([Purpose::Education, Purpose::PersonalName]),
            )),
        ),
        (
            "\"0\"",
            Some(HanjaReadingCitation(CharacterSet::KS_X_1001, BTreeSet::new())),
        ),
        (
            "\"1\"",
            Some(HanjaReadingCitation(CharacterSet::KS_X_1002, BTreeSet::new())),
        ),
    ];

    for (raw, expected) in cases {
        let parsed: Option<HanjaReadingCitation> = serde_json::from_str(raw).ok();
        assert_eq!(parsed, expected, "case: {raw}");
    }

    for raw in ["\"2\"", "\"00\"", "\"0Z\"", "0", "null"] {
        let parsed: Option<HanjaReadingCitation> = serde_json::from_str(raw).ok();
        assert_eq!(parsed, None, "case: {raw}");
    }
}
