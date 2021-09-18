import React = require("react");
import { useState, useEffect, useRef } from "react";

import { useAuth } from "../useAuth";

export default function Login() {
  const auth = useAuth();

  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(false);
  const isMountedRef = useRef(null);

  useEffect(() => {
    isMountedRef.current = true;
    return () => (isMountedRef.current = false);
  });

  const handleSubmit = async (evt) => {
    evt.preventDefault();

    setError("");
    setLoading(true);

    try {
      await auth.login(username, password);
    } catch (e) {
      setError(e.message);
    } finally {
      if (isMountedRef.current) {
        setLoading(false);
      }
    }
  };

  return (
    <div className="bg-primary p-4 text-white min-vh-100 d-flex justify-content-center align-items-center">
      <div>
        <h1>chartered ✈️</h1>
        <h6>a private, authenticated cargo registry</h6>

        <div
          className="card border-0 shadow-sm text-black p-2"
          style={{ width: "40rem" }}
        >
          <div className="card-body">
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

            <form onSubmit={handleSubmit}>
              <div className="mb-3">
                <label htmlFor="username" className="form-label">
                  Username
                </label>
                <input
                  type="text"
                  className="form-control"
                  id="username"
                  disabled={loading}
                  value={username}
                  onChange={(e) => setUsername(e.target.value)}
                />
              </div>
              <div className="mb-3">
                <label htmlFor="password" className="form-label">
                  Password
                </label>
                <input
                  type="password"
                  className="form-control"
                  id="password"
                  disabled={loading}
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                />
              </div>
              <div className="ml-auto">
                <button
                  type="submit"
                  className="btn btn-primary"
                  style={{ display: !loading ? "block" : "none" }}
                >
                  Login
                </button>

                <div
                  className="spinner-border text-primary mt-4"
                  role="status"
                  style={{ display: loading ? "block" : "none" }}
                >
                  <span className="visually-hidden">Logging in...</span>
                </div>
              </div>
            </form>
          </div>
        </div>
      </div>
    </div>
  );
}
