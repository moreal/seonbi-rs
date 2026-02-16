import { Button } from "react-bootstrap";

interface TransformButtonProps {
  wasmReady: boolean;
  wasmError: string | null;
  sourceUnchanged: boolean;
  error: string | null;
  onTransform: () => void;
}

export function TransformButton({
  wasmReady,
  wasmError,
  sourceUnchanged,
  error,
  onTransform,
}: TransformButtonProps) {
  const disabled = !wasmReady || sourceUnchanged;

  let label: string;
  if (wasmError) {
    label = `(Error: ${wasmError})`;
  } else if (!wasmReady) {
    label = "(Loading WASM...)";
  } else if (error) {
    label = `(Error: ${error})`;
  } else {
    label = "Transform";
  }

  return (
    <Button
      variant={error || wasmError ? "secondary" : "primary"}
      size="lg"
      className="w-100"
      disabled={disabled}
      onClick={onTransform}
    >
      {label}
    </Button>
  );
}
