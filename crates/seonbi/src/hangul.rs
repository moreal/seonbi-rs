pub type JamoTriple = (char, char, Option<char>);

const SYLLABLE_BASE: u32 = 0xAC00;
const INITIAL_BASE: u32 = 0x1100;
const VOWEL_BASE: u32 = 0x1161;
const FINAL_BASE: u32 = 0x11A7;
const VOWEL_COUNT: u32 = 21;
const FINAL_COUNT: u32 = 28;

pub fn is_hangul_syllable(c: char) -> bool {
    ('\u{AC00}'..='\u{D7A3}').contains(&c)
}

pub fn to_jamo_triple(c: char) -> Option<JamoTriple> {
    if !is_hangul_syllable(c) {
        return None;
    }

    let syllable = c as u32 - SYLLABLE_BASE;
    let initial = char::from_u32(INITIAL_BASE + ((syllable / FINAL_COUNT) / VOWEL_COUNT))?;
    let vowel = char::from_u32(VOWEL_BASE + ((syllable / FINAL_COUNT) % VOWEL_COUNT))?;
    let final_jamo = match syllable % FINAL_COUNT {
        0 => None,
        f => char::from_u32(FINAL_BASE + f),
    };
    Some((initial, vowel, final_jamo))
}

pub fn from_jamo_triple((initial, vowel, final_jamo): JamoTriple) -> Option<char> {
    let initial_index = initial as i32 - INITIAL_BASE as i32;
    let vowel_index = vowel as i32 - VOWEL_BASE as i32;
    let final_index = final_jamo
        .map(|f| f as i32 - FINAL_BASE as i32)
        .unwrap_or(0);

    if !(0..=18).contains(&initial_index)
        || !(0..=20).contains(&vowel_index)
        || !(0..=27).contains(&final_index)
    {
        return None;
    }

    let code = SYLLABLE_BASE
        + ((initial_index as u32 * VOWEL_COUNT + vowel_index as u32) * FINAL_COUNT)
        + final_index as u32;
    char::from_u32(code)
}
