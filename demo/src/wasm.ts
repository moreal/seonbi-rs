import wasmInit, {
  transform as wasmTransform,
  koKr as wasmKoKr,
  koKp as wasmKoKp,
} from "@seonbi/seonbi-wasm";
import type { Configuration } from "@seonbi/seonbi-wasm";

let initialized = false;

export async function init(): Promise<void> {
  if (!initialized) {
    await wasmInit();
    initialized = true;
  }
}

export function transform(config: Configuration, input: string): string {
  return wasmTransform(config, input);
}

export function koKr(): Configuration {
  return wasmKoKr();
}

export function koKp(): Configuration {
  return wasmKoKp();
}

export type { Configuration };
