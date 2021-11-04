import { SyntheticEvent, useState } from "react";
import { Link, useNavigate } from "react-router-dom";

import Nav from "../../sections/Nav";
import { useAuth } from "../../useAuth";
import { authenticatedEndpoint } from "../../util";

export default function CreateOrganisation() {
  const auth = useAuth();
  const navigate = useNavigate();

  if (!auth) {
    return <></>;
  }

  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");

  const [name, setName] = useState("");
  const [description, setDescription] = useState("");
  const [publicOrg, setPublicOrg] = useState(false);

  const createOrganisation = async (evt: SyntheticEvent) => {
    evt.preventDefault();

    setError("");
    setLoading(true);

    try {
      let res = await fetch(authenticatedEndpoint(auth, "organisations"), {
        method: "PUT",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({ name, description, public: publicOrg }),
      });
      let json = await res.json();

      if (json.error) {
        throw new Error(json.error);
      }

      setName("");
      setDescription("");
      navigate(`/crates/${name}`);
    } catch (e: any) {
      setError(e.message);
      setLoading(false);
    }
  };

  return (
    <div>
      <Nav />

      <div className="container mt-4 pb-4">
        <h1>Create New Organisation</h1>

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

        <div className="card border-0 shadow-sm text-black">
          <div className="card-body">
            <form onSubmit={createOrganisation}>
              <div className="mb-3">
                <label htmlFor="org-name" className="form-label">
                  Name
                </label>
                <input
                  id="org-name"
                  type="text"
                  className="form-control"
                  pattern="[a-zA-Z0-9-]*"
                  placeholder="backend-team"
                  onChange={(e) => setName(e.target.value)}
                  disabled={loading}
                  value={name}
                />
                <div className="form-text">
                  Must be in the format <code>[a-zA-Z0-9-]*</code>
                </div>
              </div>

              <div>
                <label htmlFor="org-description" className="form-label">
                  Description
                </label>
                <textarea
                  id="org-description"
                  className="form-control"
                  rows={3}
                  onChange={(e) => setDescription(e.target.value)}
                  disabled={loading}
                  value={description}
                />
              </div>

              <div className="mt-2 form-check">
                <input
                  type="checkbox"
                  checked={publicOrg}
                  id="org-public"
                  className="form-check-input"
                  onChange={(e) => setPublicOrg(e.target.checked)}
                  disabled={loading}
                />
                <label htmlFor="org-public" className="form-check-label">
                  Give <strong>VISIBLE</strong> permission to all logged in
                  users
                </label>
              </div>

              <div className="clearfix" />

              <button
                type="submit"
                className="btn btn-success mt-2"
                style={{ display: !loading ? "block" : "none" }}
              >
                Create
              </button>
              <div
                className="spinner-border text-primary mt-4"
                role="status"
                style={{ display: loading ? "block" : "none" }}
              >
                <span className="visually-hidden">Submitting...</span>
              </div>
            </form>
          </div>
        </div>
      </div>
    </div>
  );
}
