use seonbi::{from_jamo_triple, is_hangul_syllable, to_jamo_triple};

#[test]
fn is_hangul_syllable_works() {
    assert!(is_hangul_syllable('가'));
    assert!(is_hangul_syllable('글'));
    assert!(!is_hangul_syllable('A'));
    assert!(!is_hangul_syllable('?'));
    assert!(!is_hangul_syllable('字'));
}

#[test]
fn to_jamo_triple_works() {
    assert_eq!(to_jamo_triple('가'), Some(('ᄀ', 'ᅡ', None)));
    assert_eq!(to_jamo_triple('글'), Some(('ᄀ', 'ᅳ', Some('ᆯ'))));
    assert_eq!(to_jamo_triple('를'), Some(('ᄅ', 'ᅳ', Some('ᆯ'))));
    assert_eq!(to_jamo_triple('A'), None);
    assert_eq!(to_jamo_triple('?'), None);
    assert_eq!(to_jamo_triple('字'), None);
}

#[test]
fn from_jamo_triple_works() {
    assert_eq!(from_jamo_triple(('ᄀ', 'ᅡ', None)), Some('가'));
    assert_eq!(from_jamo_triple(('ᄀ', 'ᅳ', Some('ᆯ'))), Some('글'));
    assert_eq!(from_jamo_triple(('ᄅ', 'ᅳ', Some('ᆯ'))), Some('를'));
    assert_eq!(from_jamo_triple(('ᄓ', 'ᅳ', None)), None);
    assert_eq!(from_jamo_triple(('ᄀ', 'ᅶ', None)), None);
    assert_eq!(from_jamo_triple(('ᄀ', 'ᅳ', Some('ᅡ'))), None);
    assert_eq!(from_jamo_triple(('ᄀ', 'ᅳ', Some('ᇇ'))), None);
}
