# Custom Dictionary Support in WASM and Node.js Bindings

## Current State

- **Python binding**: Already supports custom dictionaries via `HanjaReadingOption.dictionary: dict[str, str]`
- **WASM binding** (`crates/seonbi-wasm`): `HanjaReadingOption` only has `initialSoundLaw` and `useDictionaries`, no `customDictionary` field
- **Node.js binding** (`crates/seonbi-node`): Same gap as WASM

## What Needs to Be Added

### WASM Binding (`crates/seonbi-wasm/src/lib.rs`)

1. Add a `dictionary` field (type: `Map<String, String>` or `JsValue`) to `HanjaReadingOption`
2. In `build_configuration`, merge the custom dictionary entries into the hanja reading dictionary, similar to how `use_dictionaries` entries are merged
3. Update `seonbi_wasm.d.ts` will be auto-generated to include `dictionary: Record<string, string>` or similar

### Node.js Binding (`crates/seonbi-node/src/lib.rs`)

1. Add a `dictionary` field (type: `HashMap<String, String>`) to `HanjaReadingOption` struct
2. In `build_configuration`, merge dictionary entries the same way as the WASM binding
3. The `index.d.ts` will auto-generate to include the new field

### Demo Integration

Once custom dictionary support is added to the WASM binding:

1. Remove the `disabled` prop and tooltip from the "Custom dictionary" button in `OptionsPanel.tsx`
2. Implement `CustomDictionaryModal.tsx` with:
   - A textarea for entering `漢字語 → 한글` pairs (one per line)
   - Regex parsing of `key → value` pairs (matching the Elm demo's logic)
   - Warning that data is lost on page refresh
3. Store `customDictionary: Map<string, string>` and `customDictionarySource: string` in `AppState`
4. Pass the dictionary to the WASM `Configuration.hanja.reading.dictionary` field
5. Show the dictionary entry count on the button label
