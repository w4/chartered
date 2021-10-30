import {
  useState,
  useEffect,
  useRef,
  SyntheticEvent,
  MouseEventHandler,
} from "react";
import { useLocation, Link, Redirect } from "react-router-dom";

import { useAuth } from "../useAuth";
import { unauthenticatedEndpoint, useUnauthenticatedRequest } from "../util";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { IconDefinition } from "@fortawesome/free-brands-svg-icons";
import { faSignInAlt } from "@fortawesome/free-solid-svg-icons";

interface OAuthProviders {
  providers: string[];
}

export default function Register() {
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const [error, setError] = useState("");
  const [loading, setLoading] = useState<boolean>(false);
  const [complete, setComplete] = useState<boolean>(false);

  const handleSubmit = async (evt: SyntheticEvent) => {
    evt.preventDefault();

    setError("");
    setLoading(true);

    try {
      let res = await fetch(unauthenticatedEndpoint("auth/register/password"), {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          "User-Agent": window.navigator.userAgent,
        },
        body: JSON.stringify({ username, password }),
      });
      let json = await res.json();

      if (json.error) {
        throw new Error(json.error);
      } else if (!json.success) {
        throw new Error("Failed to register, please try again later.");
      } else {
        setComplete(true);
      }
    } catch (e: any) {
      setError(e.message);
    } finally {
      setLoading(false);
    }
  };

  if (complete) {
    return (
      <Redirect
        to={{
          pathname: "/login",
          state: {
            prompt: {
              message: "Successfully registered, please login.",
              kind: "success",
            },
          },
        }}
      />
    );
  }

  return (
    <div className="p-4 min-vh-100 d-flex justify-content-center align-items-center">
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
              />
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

                <label htmlFor="username" className="form-label">
                  Username
                </label>
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

                <label htmlFor="password" className="form-label">
                  Password
                </label>
              </div>

              <ButtonOrSpinner
                type="submit"
                variant="primary"
                disabled={loading}
                showSpinner={loading}
                text={`Register`}
                icon={faSignInAlt}
                onClick={handleSubmit}
              />
            </form>
          </div>
        </div>
      </div>
    </div>
  );
}

function ButtonOrSpinner({
  type,
  variant,
  disabled,
  showSpinner,
  text,
  icon,
  background,
  onClick,
}: {
  type: "button" | "submit";
  variant: string;
  disabled: boolean;
  showSpinner: boolean;
  text: string;
  icon?: IconDefinition;
  background?: string;
  onClick: MouseEventHandler<HTMLButtonElement>;
}) {
  if (showSpinner) {
    return (
      <div
        className="spinner-border text-primary mt-3 m-auto d-block"
        role="status"
      >
        <span className="visually-hidden">Logging in...</span>
      </div>
    );
  }

  if (type) {
    return (
      <button
        type={type}
        disabled={disabled}
        onClick={onClick}
        className={`btn btn-lg mt-2 btn-${variant} w-100`}
        style={{ background, borderColor: background }}
      >
        {icon ? <FontAwesomeIcon icon={icon} className="me-2" /> : <></>}
        {text}
      </button>
    );
  }

  return <></>;
}
