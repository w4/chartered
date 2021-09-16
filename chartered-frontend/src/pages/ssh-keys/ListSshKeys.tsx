import React = require("react");
import { useState, useEffect } from 'react';
import { Link } from 'react-router-dom';

import Nav from "../../sections/Nav";
import { useAuth } from "../../useAuth";
import { authenticatedEndpoint } from "../../util";

import { Plus, Trash } from "react-bootstrap-icons";
import { Button, Modal } from "react-bootstrap";

export default function ListSshKeys() {
    const auth = useAuth();

    const [error, setError] = useState("");
    const [deleting, setDeleting] = useState(null);
    const [sshKeys, setSshKeys] = useState(null);
    const [reloadSshKeys, setReloadSshKeys] = useState(0);
    useEffect(async () => {
        let res = await fetch(authenticatedEndpoint(auth, 'ssh-key'));
        let json = await res.json();
        setSshKeys(json);
    }, [reloadSshKeys]);

    if (!sshKeys) {
        return (<div>loading...</div>);
    }

    const deleteKey = async () => {
        setError("");

        try {
            let res = await fetch(authenticatedEndpoint(auth, `ssh-key/${deleting.id}`), {
                method: 'DELETE',
                headers: {
                    'Content-Type': 'application/json',
                },
            });
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

    return (
        <div className="text-white">
            <Nav />

            <div className="container mt-4 pb-4">
                <h1>Manage your SSH Keys</h1>

                <div className="alert alert-danger alert-dismissible" role="alert" style={{ display: error ? 'block' : 'none' }}>
                    {error}

                    <button type="button" className="btn-close" aria-label="Close" onClick={() => setError("")}>
                    </button>
                </div>

                <div className="card border-0 shadow-sm text-black">
                    <ul className="list-group list-group-flush">
                        {sshKeys.keys.map(key => (
                            <li key={key.id} className="list-group-item">
                                <div className="d-flex">
                                    <div className="flex-grow-1"><strong>{key.fingerprint}</strong></div>
                                    <div>
                                        <button type="button" className="btn text-danger" onClick={() => setDeleting(key)}><Trash /></button>
                                    </div>
                                </div>
                            </li>
                        ))}
                    </ul>
                </div>

                <Link to="/ssh-keys/add" className="btn btn-outline-light mt-2 float-end"><Plus /> Add New</Link>
            </div>

            <DeleteModal show={deleting != null}
                onCancel={() => setDeleting(null)}
                onConfirm={() => deleteKey()}
                fingerprint={deleting?.fingerprint} />
        </div>
    );
}

function DeleteModal(props: { show: boolean, onCancel: () => void, onConfirm: () => void, fingerprint: string }) {
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
            Are you sure you wish to delete the SSH key with the fingerprint: <strong>{props.fingerprint}</strong>?
          </p>
        </Modal.Body>
        <Modal.Footer>
          <Button onClick={props.onCancel} variant="primary">Close</Button>
          <Button onClick={props.onConfirm} variant="danger">Delete</Button>
        </Modal.Footer>
      </Modal>
    );
  }
