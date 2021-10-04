import React = require("react");
import { useState, useEffect, useRef } from "react";

import { useAuth } from "../useAuth";
import { useUnauthenticatedRequest } from "../util";

export default function Login() {
  const auth = useAuth();

  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(false);
  const isMountedRef = useRef(null);

  const { response: oauthProviders } = useUnauthenticatedRequest({ endpoint: "login/oauth/providers" });

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
              <div className="form-floating">
                <input
                  type="text"
                  className="form-control"
                  placeholder="john.smith"
                  id="username"
                  disabled={loading}
                  value={username}
                  onChange={(e) => setUsername(e.target.value)}
                />

                <label htmlFor="email" className="form-label">Username</label>
              </div>

              <div className="form-floating mt-2">
                <input
                  type="password"
                  className="form-control"
                  placeholder="&bull;&bull;&bull;&bull;&bull;&bull;&bull;&bull;&bull;&bull;&bull;&bull;"
                  id="password"
                  disabled={loading}
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                />

                <label htmlFor="password" className="form-label">Password</label>
              </div>

              <div className="mt-2 ml-auto">
                <button
                  type="submit"
                  className="btn btn-lg btn-primary w-100"
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

            {oauthProviders?.providers.length > 0 ? (<>
              <div className="side-lines mt-3">or</div>

              {oauthProviders.providers.map((v, i) => <a href="#" key={i} className="btn btn-lg btn-dark w-100 mt-3">Login with {v}</a>)}
            </>): <></>}
          </div>
        </div>
      </div>
    </div>
  );
}
