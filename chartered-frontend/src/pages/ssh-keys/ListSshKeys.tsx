import React = require("react");
import { useState, useEffect } from "react";
import { Link } from "react-router-dom";

import Nav from "../../sections/Nav";
import { useAuth } from "../../useAuth";
import { useAuthenticatedRequest, authenticatedEndpoint } from "../../util";

import { Plus, Trash } from "react-bootstrap-icons";
import { Button, Modal, OverlayTrigger, Tooltip } from "react-bootstrap";
import HumanTime from "react-human-time";
import ErrorPage from "../ErrorPage";
import Loading, { LoadingSpinner } from "../Loading";

interface SshKeysResponse {
  keys: SshKeysResponseKey[];
}

interface SshKeysResponseKey {
  uuid: string;
  name: string;
  fingerprint: string;
  created_at: string;
  last_used_at: string;
}

export default function ListSshKeys() {
  const auth = useAuth();

  const [error, setError] = useState("");
  const [deleting, setDeleting] = useState(null);
  const [reloadSshKeys, setReloadSshKeys] = useState(0);

  const { response: sshKeys, error: loadError } =
    useAuthenticatedRequest<SshKeysResponse>(
      {
        auth,
        endpoint: "ssh-key",
      },
      [reloadSshKeys]
    );

  if (loadError) {
    return <ErrorPage message={loadError} />;
  }

  const deleteKey = async () => {
    setError("");

    try {
      let res = await fetch(
        authenticatedEndpoint(auth, `ssh-key/${deleting.uuid}`),
        {
          method: "DELETE",
          headers: {
            "Content-Type": "application/json",
          },
        }
      );
      let json = await res.json();

      if (json.error) {
        throw new Error(json.error);
      }

      setReloadSshKeys(reloadSshKeys + 1);
    } catch (e) {
      setError(e.message);
    } finally {
      setDeleting(null);
    }
  };

  const dateMonthAgo = new Date();
  dateMonthAgo.setMonth(dateMonthAgo.getMonth() - 1);

  return (
    <div className="text-white">
      <Nav />

      <div className="container mt-4 pb-4">
        <h1>Manage your SSH Keys</h1>

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
          ></button>
        </div>

        <div className="card border-0 shadow-sm text-black">
          {!sshKeys ? (
            <LoadingSpinner />
          ) : (
            <>
              {sshKeys.keys.length == 0 ? (
                <div className="card-body">
                  You haven't added any SSH keys yet.
                </div>
              ) : (
                <div className="table-responsive">
                  <table className="table table-striped">
                    <tbody>
                      {sshKeys.keys.map((key) => (
                        <tr key={key.uuid}>
                          <td className="align-middle">
                            <h6 className="m-0 lh-sm">{key.name}</h6>
                            <pre className="m-0">{key.fingerprint}</pre>
                            <div
                              className="lh-sm"
                              style={{ fontSize: ".75rem" }}
                            >
                              <div className="text-muted d-inline-block me-3">
                                Added{" "}
                                <OverlayTrigger
                                  overlay={
                                    <Tooltip id={`${key.uuid}-created-at`}>
                                      {new Date(
                                        key.created_at
                                      ).toLocaleString()}
                                    </Tooltip>
                                  }
                                >
                                  <span className="text-decoration-underline-dotted">
                                    <HumanTime
                                      time={new Date(key.created_at).getTime()}
                                    />
                                  </span>
                                </OverlayTrigger>
                              </div>
                              <span
                                className={`text-${
                                  key.last_used_at
                                    ? new Date(key.last_used_at) > dateMonthAgo
                                      ? "success"
                                      : "danger"
                                    : "muted"
                                }`}
                              >
                                Last used{" "}
                                {key.last_used_at ? (
                                  <OverlayTrigger
                                    overlay={
                                      <Tooltip id={`${key.uuid}-last-used`}>
                                        {new Date(
                                          key.last_used_at
                                        ).toLocaleString()}
                                      </Tooltip>
                                    }
                                  >
                                    <span className="text-decoration-underline-dotted">
                                      <HumanTime
                                        time={new Date(
                                          key.last_used_at
                                        ).getTime()}
                                      />
                                    </span>
                                  </OverlayTrigger>
                                ) : (
                                  <>never</>
                                )}
                              </span>
                            </div>
                          </td>

                          <td className="align-middle fit">
                            <button
                              type="button"
                              className="btn text-danger"
                              onClick={() => setDeleting(key)}
                            >
                              <Trash />
                            </button>
                          </td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                </div>
              )}
            </>
          )}
        </div>

        <Link
          to="/ssh-keys/add"
          className="btn btn-outline-light mt-2 float-end"
        >
          <Plus /> Add New
        </Link>
      </div>

      <DeleteModal
        show={deleting != null}
        onCancel={() => setDeleting(null)}
        onConfirm={() => deleteKey()}
        fingerprint={deleting?.fingerprint}
      />
    </div>
  );
}

function DeleteModal(props: {
  show: boolean;
  onCancel: () => void;
  onConfirm: () => void;
  fingerprint: string;
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
          Are you sure you wish to delete this SSH key?
        </Modal.Title>
      </Modal.Header>
      <Modal.Body>
        <p>
          Are you sure you wish to delete the SSH key with the fingerprint:{" "}
          <strong>{props.fingerprint}</strong>?
        </p>
      </Modal.Body>
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
