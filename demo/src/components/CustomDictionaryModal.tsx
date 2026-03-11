import { Modal, Button, Form, Alert } from "react-bootstrap";
import type { Action } from "../types";

interface CustomDictionaryModalProps {
  show: boolean;
  source: string;
  entryCount: number;
  dispatch: React.Dispatch<Action>;
}

export function CustomDictionaryModal({
  show,
  source,
  entryCount,
  dispatch,
}: CustomDictionaryModalProps) {
  return (
    <Modal show={show} onHide={() => dispatch({ type: "CLOSE_CUSTOM_DICTIONARY" })} size="lg">
      <Modal.Header closeButton>
        <Modal.Title as="h5">Custom hanja readings ({entryCount})</Modal.Title>
      </Modal.Header>
      <Modal.Body>
        <Form.Control
          as="textarea"
          rows={24}
          placeholder={
            "Sino-Korean word \u2192 Hangul reading\n\u6F22\u5B57\u8A9E \u2192 \ud55c\uc790\uc5b4"
          }
          value={source}
          onChange={(e) =>
            dispatch({
              type: "UPDATE_CUSTOM_DICTIONARY_SOURCE",
              source: e.target.value,
            })
          }
        />
        <Alert variant="warning" className="mt-3">
          This data will be gone if you refresh this page.
        </Alert>
      </Modal.Body>
      <Modal.Footer>
        <Button
          variant="outline-primary"
          onClick={() => dispatch({ type: "CLOSE_CUSTOM_DICTIONARY" })}
        >
          Close
        </Button>
      </Modal.Footer>
    </Modal>
  );
}
