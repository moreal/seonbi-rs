use std::collections::{BTreeMap, BTreeSet};
use std::str::FromStr;

use once_cell::sync::Lazy;
use serde::de::{Error, Visitor};
use serde::{Deserialize, Deserializer};

pub type KHangulData = BTreeMap<char, HanjaReadings>;
pub type HanjaReadings = BTreeMap<char, HanjaReadingCitation>;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct HanjaReadingCitation(pub CharacterSet, pub BTreeSet<Purpose>);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CharacterSet {
    KS_X_1001,
    KS_X_1002,
    NonStandard,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Purpose {
    Education,
    PersonalName,
}

impl FromStr for HanjaReadingCitation {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = s.as_bytes();
        let mut idx = 0usize;
        let charset = if let Some(first) = bytes.first().copied() {
            match first as char {
                '0' => {
                    idx = 1;
                    CharacterSet::KS_X_1001
                }
                '1' => {
                    idx = 1;
                    CharacterSet::KS_X_1002
                }
                'X' => {
                    idx = 1;
                    CharacterSet::NonStandard
                }
                'E' | 'N' => CharacterSet::NonStandard,
                _ => return Err(format!("Invalid kHangul character set code: {}", first as char)),
            }
        } else {
            CharacterSet::NonStandard
        };

        let mut purposes = BTreeSet::new();
        for c in s[idx..].chars() {
            match c {
                'E' => {
                    purposes.insert(Purpose::Education);
                }
                'N' => {
                    purposes.insert(Purpose::PersonalName);
                }
                // Some entries in the bundled kHangul dataset use "X" as an
                // opaque marker. We preserve compatibility by accepting it.
                'X' => {}
                _ => return Err(format!("Invalid kHangul purpose code: {}", c)),
            }
        }

        Ok(HanjaReadingCitation(charset, purposes))
    }
}

impl<'de> Deserialize<'de> for HanjaReadingCitation {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct CitationVisitor;

        impl Visitor<'_> for CitationVisitor {
            type Value = HanjaReadingCitation;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("kHangul value (e.g., 0E, 1N, 0EN)")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: Error,
            {
                v.parse().map_err(Error::custom)
            }
        }

        deserializer.deserialize_str(CitationVisitor)
    }
}

pub static K_HANGUL_DATA_RESULT: Lazy<Result<KHangulData, String>> = Lazy::new(|| {
    let raw: BTreeMap<String, BTreeMap<String, HanjaReadingCitation>> =
        serde_json::from_str(include_str!("../../data/kHangul.json")).map_err(|e| e.to_string())?;

    let mut data = BTreeMap::new();
    for (hanja_key, readings) in raw {
        let mut hanja_chars = hanja_key.chars();
        let hanja = hanja_chars.next().ok_or_else(|| "empty hanja key".to_string())?;
        if hanja_chars.next().is_some() {
            return Err(format!("invalid hanja key: {hanja_key}"));
        }

        let mut converted = BTreeMap::new();
        for (reading_key, citation) in readings {
            let mut chars = reading_key.chars();
            let reading = chars
                .next()
                .ok_or_else(|| "empty reading key".to_string())?;
            if chars.next().is_some() {
                return Err(format!("invalid reading key: {reading_key}"));
            }
            converted.insert(reading, citation);
        }

        data.insert(hanja, converted);
    }

    Ok(data)
});

pub fn k_hangul_data_result() -> &'static Result<KHangulData, String> {
    &K_HANGUL_DATA_RESULT
}

pub fn k_hangul_data() -> &'static KHangulData {
    static EMPTY: Lazy<KHangulData> = Lazy::new(BTreeMap::new);
    match &*K_HANGUL_DATA_RESULT {
        Ok(data) => data,
        Err(_) => &EMPTY,
    }
}
