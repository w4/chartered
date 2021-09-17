import React = require("react");
import { useState } from "react";
import { PersonPlus, Trash, CheckLg, Save, PlusLg } from 'react-bootstrap-icons';
import { authenticatedEndpoint, useAuthenticatedRequest } from "../../util";
import { useAuth } from "../../useAuth";
import { Button, Modal } from "react-bootstrap";

interface CratesMembersResponse {
    allowed_permissions: string[],
    members: Member[],
}

interface Member {
    id: number,
    username: string,
    permissions: string[],
}

export default function Members({ crate }: { crate: string }) {
    const auth = useAuth();
    const [reload, setReload] = useState(0);
    const { response, error } = useAuthenticatedRequest<CratesMembersResponse>({
        auth,
        endpoint: `crates/${crate}/members`,
    }, [reload]);

    if (error) {
        return <>{error}</>;
    } else if (!response) {
        return <div className="d-flex justify-content-center align-items-center">
            <div className="spinner-border text-light" role="status">
                <span className="visually-hidden">Loading...</span>
            </div>
        </div>;
    }

    const allowedPermissions = response.allowed_permissions;

    return <div className="container-fluid g-0">
        <div className="table-responsive">
            <table className="table table-striped">
                <tbody>
                    {response.members.map((member, index) =>
                        <MemberListItem
                            key={index}
                            crate={crate}
                            member={member}
                            allowedPermissions={allowedPermissions}
                            onUpdateComplete={() => setReload(reload + 1)}
                        />
                    )}

                    <tr>
                        <td className="align-middle fit">
                            <div
                                className="d-flex align-items-center justify-content-center rounded-circle"
                                style={{ width: '48px', height: '48px', background: '#DEDEDE', fontSize: '1rem' }}
                            >
                                <PersonPlus />
                            </div>
                        </td>

                        <td className="align-middle">
                            <input type="search" className="form-control" placeholder="Search for User" />
                        </td>

                        <td className="align-middle">
                            <RenderPermissions allowedPermissions={allowedPermissions} selectedPermissions={[]} userId={-1} />
                        </td>

                        <td className="align-middle">
                            <button type="button" className="btn text-dark pe-none">
                                <PlusLg />
                            </button>
                        </td>
                    </tr>
                </tbody>
            </table>
        </div>
    </div>;
}

function MemberListItem({ crate, member, allowedPermissions, onUpdateComplete }: { crate: string, member: Member, allowedPermissions: string[], onUpdateComplete: () => any }) {
    const auth = useAuth();
    const [selectedPermissions, setSelectedPermissions] = useState(member.permissions);
    const [deleting, setDeleting] = useState(false);
    const [saving, setSaving] = useState(false);
    const [error, setError] = useState(null);

    let itemAction = <></>;

    const saveUserPermissions = async () => {
        setSaving(true);

        try {
            let res = await fetch(authenticatedEndpoint(auth, `crates/${crate}/members`), {
                method: 'PATCH',
                headers: {
                    'Accept': 'application/json',
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({
                    user_id: member.id,
                    permissions: selectedPermissions,
                }),
            });
            let json = await res.json();

            if (json.error) {
                throw new Error(json.error);
            }

            onUpdateComplete();
        } catch (e) {
            setError(error);
        } finally {
            setSaving(false);
        }
    };

    const doDelete = async () => {
        setSaving(true);

        try {
            let res = await fetch(authenticatedEndpoint(auth, `crates/${crate}/members`), {
                method: 'DELETE',
                headers: {
                    'Accept': 'application/json',
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({
                    user_id: member.id,
                }),
            });
            let json = await res.json();

            if (json.error) {
                throw new Error(json.error);
            }

            onUpdateComplete();
        } catch (e) {
            setError(error);
        } finally {
            setSaving(false);
        }
    };

    if (saving) {
        itemAction = <button type="button" className="btn">
            <div className="spinner-grow spinner-grow-sm text-primary" role="status">
                <span className="visually-hidden">Loading...</span>
            </div>
        </button>;
    } else if (selectedPermissions.indexOf("VISIBLE") === -1) {
        itemAction = <button type="button" className="btn text-danger" onClick={() => setDeleting(true)}>
            <Trash />
        </button>;
    } else if (selectedPermissions.sort().join(',') != member.permissions.sort().join(',')) {
        itemAction = <button type="button" className="btn text-success" onClick={saveUserPermissions}>
            <CheckLg />
        </button>;
    }

    return <>
        <DeleteModal show={deleting === true}
            onCancel={() => setDeleting(false)}
            onConfirm={() => doDelete()}
            username={member.username} />

        <ErrorModal error={error} onClose={() => setError(null)} />

        <tr>
            <td className="align-middle fit">
                <img src="http://placekitten.com/48/48" className="rounded-circle" />
            </td>

            <td className="align-middle">
                <strong>{member.username}</strong><br />
                <em>(that's you!)</em>
            </td>

            <td className="align-middle">
                <RenderPermissions
                    allowedPermissions={allowedPermissions}
                    selectedPermissions={selectedPermissions}
                    userId={member.id}
                    onChange={setSelectedPermissions}
                />
            </td>

            <td className="align-middle fit">
                {itemAction}
            </td>
        </tr>
    </>;
}

function RenderPermissions({ allowedPermissions, selectedPermissions, userId, onChange }: { allowedPermissions: string[], selectedPermissions: string[], userId: number, onChange: (permissions) => any }) {
    return (
        <div className="row ms-2">
            {allowedPermissions.map((permission) => (
                <div key={permission + userId} className="form-check col-12 col-md-6">
                    <input
                        className="form-check-input"
                        type="checkbox"
                        value="1"
                        id={`checkbox-${userId}-${permission}`}
                        checked={selectedPermissions.indexOf(permission) > -1}
                        onChange={(e) => {
                            let newUserPermissions = new Set(selectedPermissions);

                            if (e.target.checked) {
                                newUserPermissions.add(permission);
                            } else {
                                newUserPermissions.delete(permission);
                            }

                            onChange(Array.from(newUserPermissions));
                        }}
                    />
                    <label className="form-check-label" htmlFor={`checkbox-${userId}-${permission}`}>
                        {permission}
                    </label>
                </div>
            ))}
        </div>
    );
}

function DeleteModal(props: { show: boolean, onCancel: () => void, onConfirm: () => void, username: string }) {
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
                    Are you sure you wish to remove this member from the crate?
                </Modal.Title>
            </Modal.Header>
            <Modal.Body>
                <p>
                Are you sure you wish to remove <strong>{props.username}</strong> from the crate?
                </p>
            </Modal.Body>
            <Modal.Footer>
                <Button onClick={props.onCancel} variant="primary">Close</Button>
                <Button onClick={props.onConfirm} variant="danger">Delete</Button>
            </Modal.Footer>
        </Modal>
    );
}

function ErrorModal(props: { error?: string, onClose: () => void }) {
    return (
        <Modal
            show={props.error != null}
            onHide={props.onClose}
            size="lg"
            aria-labelledby="error-modal-title"
            centered
        >
            <Modal.Header closeButton>
                <Modal.Title id="error-modal-title">
                    Error
                </Modal.Title>
            </Modal.Header>
            <Modal.Body>
                <p>
                    {props.error}
                </p>
            </Modal.Body>
            <Modal.Footer>
                <Button onClick={props.onClose} variant="primary">Close</Button>
            </Modal.Footer>
        </Modal>
    );
}
