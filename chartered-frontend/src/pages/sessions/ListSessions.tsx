import Nav from "../../sections/Nav";
import { useAuth } from "../../useAuth";
import { Link, Navigate } from "react-router-dom";
import {
  authenticatedEndpoint,
  RoundedPicture,
  useAuthenticatedRequest,
} from "../../util";
import ErrorPage from "../ErrorPage";
import { LoadingSpinner } from "../Loading";
import { Button, Modal, OverlayTrigger, Tooltip } from "react-bootstrap";
import HumanTime from "react-human-time";
import { XOctagonFill } from "react-bootstrap-icons";
import { useState } from "react";

interface Response {
  sessions: ResponseSession[];
}

interface ResponseSession {
  uuid: string;
  expires_at: string | null;
  user_agent: string | null;
  ip: string | null;
  ssh_key_fingerprint: string | null;
}

export default function ListSessions() {
  const auth = useAuth();
  const [deleting, setDeleting] = useState<null | string>(null);
  const [error, setError] = useState("");
  const [reloadSessions, setReloadSessions] = useState(0);

  if (!auth) {
    return <Navigate to="/login" />;
  }

  const { response: list, error: loadError } =
    useAuthenticatedRequest<Response>(
      {
        auth,
        endpoint: "sessions",
      },
      [reloadSessions]
    );

  if (loadError) {
    return <ErrorPage message={loadError} />;
  }

  const deleteSession = async () => {
    setError("");

    if (!deleting) {
      return;
    }

    try {
      let res = await fetch(authenticatedEndpoint(auth, "sessions"), {
        method: "DELETE",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({ uuid: deleting }),
      });
      let json = await res.json();

      if (json.error) {
        throw new Error(json.error);
      }

      setReloadSessions(reloadSessions + 1);
    } catch (e: any) {
      setError(e.message);
    } finally {
      setDeleting(null);
    }
  };

  return (
    <div>
      <Nav />

      <div className="container mt-4 pb-4">
        <h1>Active Sessions</h1>

        <div
          className="alert alert-danger alert-dismissible"
          role="alert"
          style={{ display: error ? "block" : "none" }}
        >
          {error}

          <button
            type="button"
            className="btn-close"
            aria-label="Close"
            onClick={() => setError("")}
          />
        </div>

        <div className="card border-0 shadow-sm text-black p-2">
          {!list ? (
            <LoadingSpinner />
          ) : (
            <>
              {list.sessions.length === 0 ? (
                <div className="card-body">
                  You don't belong to any organisations yet.
                </div>
              ) : (
                <table className="table table-borderless mb-0">
                  <thead>
                    <tr>
                      <th scope="col">IP Address</th>
                      <th scope="col">User Agent</th>
                      <th scope="col">SSH Key Fingerprint</th>
                      <th scope="col">Expires</th>
                      <th scope="col"></th>
                    </tr>
                  </thead>

                  <tbody>
                    {list.sessions.map((v, i) => (
                      <tr key={i}>
                        <td>
                          <strong>{v.ip?.split(":")[0]}</strong>
                        </td>
                        <td>{v.user_agent}</td>
                        <td>{v.ssh_key_fingerprint || "n/a"}</td>
                        <td>
                          {v.expires_at ? (
                            <OverlayTrigger
                              overlay={
                                <Tooltip id={`sessions-${i}-created-at`}>
                                  {new Date(v.expires_at).toLocaleString()}
                                </Tooltip>
                              }
                            >
                              <span className="text-decoration-underline-dotted">
                                <HumanTime
                                  time={new Date(v.expires_at).getTime()}
                                />
                              </span>
                            </OverlayTrigger>
                          ) : (
                            "n/a"
                          )}
                        </td>
                        <td className="fit">
                          <button
                            type="button"
                            className="btn text-danger p-0"
                            onClick={() => setDeleting(v.uuid)}
                          >
                            <XOctagonFill />
                          </button>
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              )}
            </>
          )}
        </div>
      </div>

      <DeleteModal
        show={deleting !== null}
        onCancel={() => setDeleting(null)}
        onConfirm={() => deleteSession()}
      />
    </div>
  );
}

function DeleteModal(props: {
  show: boolean;
  onCancel: () => void;
  onConfirm: () => void;
}) {
  return (
    <Modal
      show={props.show}
      onHide={props.onCancel}
      size="lg"
      aria-labelledby="delete-modal-title"
      centered
    >
      <Modal.Header closeButton>
        <Modal.Title id="delete-modal-title">
          Are you sure you wish to delete this session?
        </Modal.Title>
      </Modal.Header>
      <Modal.Footer>
        <Button onClick={props.onCancel} variant="primary">
          Close
        </Button>
        <Button onClick={props.onConfirm} variant="danger">
          Delete
        </Button>
      </Modal.Footer>
    </Modal>
  );
}
