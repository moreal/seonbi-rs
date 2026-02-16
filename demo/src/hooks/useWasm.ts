import { useState, useEffect } from "react";
import { init } from "../wasm";

export function useWasm(): { ready: boolean; error: string | null } {
  const [ready, setReady] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    init()
      .then(() => setReady(true))
      .catch((err) => setError(String(err)));
  }, []);

  return { ready, error };
}
