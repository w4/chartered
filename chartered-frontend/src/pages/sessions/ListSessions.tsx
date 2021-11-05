import Nav from "../../sections/Nav";
import { useAuth } from "../../useAuth";
import { Link, Navigate } from "react-router-dom";
import { RoundedPicture, useAuthenticatedRequest } from "../../util";
import ErrorPage from "../ErrorPage";
import { LoadingSpinner } from "../Loading";
import { OverlayTrigger, Tooltip } from "react-bootstrap";
import HumanTime from "react-human-time";

interface Response {
  sessions: ResponseSession[];
}

interface ResponseSession {
  expires_at: string | null;
  user_agent: string | null;
  ip: string | null;
  ssh_key_fingerprint: string | null;
}

export default function ListSessions() {
  const auth = useAuth();

  if (!auth) {
    return <Navigate to="/login" />;
  }

  const { response: list, error } = useAuthenticatedRequest<Response>({
    auth,
    endpoint: "sessions",
  });

  if (error) {
    return <ErrorPage message={error} />;
  }

  return (
    <div>
      <Nav />

      <div className="container mt-4 pb-4">
        <h1>Active Sessions</h1>

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
                      </tr>
                    ))}
                  </tbody>
                </table>
              )}
            </>
          )}
        </div>
      </div>
    </div>
  );
}
