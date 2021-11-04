import { useParams, Navigate } from "react-router-dom";
import { useAuth } from "../useAuth";
import { ProfilePicture, useAuthenticatedRequest } from "../util";
import Nav from "../sections/Nav";
import ErrorPage from "./ErrorPage";
import ReactPlaceholder from "react-placeholder/lib";
import { Envelope, HouseDoor } from "react-bootstrap-icons";

interface Response {
  uuid: string;
  username: string;
  name?: string;
  nick?: string;
  email?: string;
  external_profile_url?: string;
  picture_url?: string;
}

interface UrlParams {
  uuid: string;
}

export default function User() {
  const auth = useAuth();
  const { uuid } = useParams();

  if (!auth) {
    return <Navigate to="/login" />;
  }

  const { response: user, error } = useAuthenticatedRequest<Response>({
    auth,
    endpoint: "users/info/" + uuid,
  });

  if (error) {
    return <ErrorPage message={error} />;
  }

  const ready = !!user;

  return (
    <div>
      <Nav />

      <div className="container mt-4 pb-4">
        <div className="row align-items-stretch">
          <div className="col-12 col-md-6 mb-3">
            <div className="card border-0 shadow-sm text-black h-100">
              <div className="card-body">
                <div className="d-flex flex-row align-items-center">
                  <ProfilePicture
                    src={user?.picture_url}
                    height="96px"
                    width="96px"
                  />

                  <div className="px-2">
                    <h1 className="text-primary my-0">
                      <ReactPlaceholder
                        showLoadingAnimation
                        type="text"
                        rows={1}
                        ready={ready}
                        style={{ width: "12rem" }}
                      >
                        {user?.nick || user?.name || user?.username}
                      </ReactPlaceholder>
                    </h1>
                  </div>
                </div>
              </div>
            </div>
          </div>

          <div className="col-12 col-md-6 mb-3">
            <div className="card border-0 shadow-sm text-black h-100">
              <div className="card-body">
                <h5>Aliases</h5>

                {user?.nick ? (
                  <>
                    {user?.nick}
                    <br />
                  </>
                ) : (
                  <></>
                )}
                {user?.name ? (
                  <>
                    {user?.name}
                    <br />
                  </>
                ) : (
                  <></>
                )}
                {user?.username ? (
                  <>
                    {user?.username}
                    <br />
                  </>
                ) : (
                  <></>
                )}
              </div>
            </div>
          </div>
        </div>

        <div className="row align-items-stretch">
          <div className="col-12">
            <div className="card border-0 shadow-sm text-black h-100">
              <div className="card-body">
                <h5>Links</h5>

                {ready && !user?.external_profile_url && !user?.email
                  ? `${
                      user?.nick || user?.name || user?.username
                    } hasn't added any links to their profile yet.`
                  : ""}

                {!ready || user?.external_profile_url ? (
                  <div>
                    <HouseDoor />
                    &nbsp;
                    <ReactPlaceholder
                      showLoadingAnimation
                      type="text"
                      rows={1}
                      ready={ready}
                      className="position-relative d-inline-block"
                      style={{ width: "12rem", top: "4px" }}
                    >
                      <a href={user?.external_profile_url} target="_blank">
                        {user?.external_profile_url}
                      </a>
                    </ReactPlaceholder>
                  </div>
                ) : (
                  <></>
                )}

                {!ready || user?.email ? (
                  <div>
                    <Envelope />
                    &nbsp;
                    <ReactPlaceholder
                      showLoadingAnimation
                      type="text"
                      rows={1}
                      ready={ready}
                      className="position-relative d-inline-block"
                      style={{ width: "12rem", top: "4px" }}
                    >
                      <a href={`mailto:${user?.email}`}>{user?.email}</a>
                    </ReactPlaceholder>
                  </div>
                ) : (
                  <></>
                )}
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
