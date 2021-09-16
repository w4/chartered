import React = require("react");
import { useState, useEffect } from 'react';
import { Link, useHistory } from 'react-router-dom';

import Nav from "../../sections/Nav";
import { useAuth } from "../../useAuth";
import { authenticatedEndpoint } from "../../util";

import { Plus } from "react-bootstrap-icons";

export default function ListSshKeys() {
    const auth = useAuth();
    const router = useHistory();

    const [sshKey, setSshKey] = useState("");
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState("");

    const submitSshKey = async (evt) => {
        evt.preventDefault();

        setError("");
        setLoading(true);

        try {
            let res = await fetch(authenticatedEndpoint(auth, 'ssh-key'), {
                method: 'PUT',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({ key: sshKey }),
            });
            let json = await res.json();

            if (json.error) {
                throw new Error(json.error);
            }

            setSshKey("");
            router.push("/ssh-keys/list");
        } catch (e) {
            setError(e.message);
        } finally {
            setLoading(false);
        }
    };

    return (
        <div className="text-white">
            <Nav />

            <div className="container mt-4 pb-4">
                <h1>New SSH Key</h1>

                <div className="alert alert-danger alert-dismissible" role="alert" style={{ display: error ? 'block' : 'none' }}>
                    {error}

                    <button type="button" className="btn-close" aria-label="Close" onClick={() => setError("")}>
                    </button>
                </div>

                <div className="card border-0 shadow-sm text-black">
                    <div className="card-body">
                        <form onSubmit={submitSshKey}>
                            <textarea className="form-control" rows={3}
                                placeholder="ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAILYAIoV2OKRSh/DcM3TicD/NK/4TdqwwBPbKgFQKmGZ3 john@home"
                                onChange={e => setSshKey(e.target.value)}
                                value={sshKey}
                            />

                            <div className="clearfix"></div>

                            <button type="submit" className="btn btn-success mt-2 float-end" style={{ display: !loading ? 'block' : 'none' }}>Submit</button>
                            <div className="spinner-border text-primary mt-4 float-end" role="status" style={{ display: loading ? 'block' : 'none' }}>
                                <span className="visually-hidden">Submitting...</span>
                            </div>

                            <Link to="/ssh-keys/list" className="btn btn-danger mt-2 float-end me-1">Cancel</Link>
                        </form>
                    </div>
                </div>
            </div>
        </div>
    );
}