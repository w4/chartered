import { Link, useSearchParams } from "react-router-dom";

import Nav from "../sections/Nav";
import { useAuth } from "../useAuth";
import { ProfilePicture, useAuthenticatedRequest } from "../util";

import { BoxSeam } from "react-bootstrap-icons";
import { LoadingSpinner } from "./Loading";

export default function Search() {
  const [searchParams] = useSearchParams();

  const query = searchParams.get("q");

  return (
    <div>
      <Nav />

      <div className="container mt-4 pb-4">
        <h1>Search Results {query ? <>for '{query}'</> : <></>}</h1>

        <UsersResults query={query} />
        <CrateResults query={query} className="mt-2" />
      </div>
    </div>
  );
}

interface UsersSearchResponse {
  users: UserSearchResponseUser[];
}

interface UserSearchResponseUser {
  user_uuid: string;
  display_name: string;
  picture_url: string;
}

function UsersResults({ query }: { query: string }) {
  const auth = useAuth();

  if (!auth) {
    return <></>;
  }

  const { response: results, error } =
    useAuthenticatedRequest<UsersSearchResponse>(
      {
        auth,
        endpoint: "users/search?q=" + encodeURIComponent(query),
      },
      [query]
    );

  if (error) {
    return <div className="alert alert-danger">{error}</div>;
  }

  if (!results) {
    return (
      <div className="card border-0 shadow-sm text-black p-2">
        <div className="card-body">
          {[0, 1, 2].map((i) => (
            <ProfilePicture
              key={i}
              height="5rem"
              width="5rem"
              className="me-2"
              src={undefined}
            />
          ))}
        </div>
      </div>
    );
  }

  if (results?.users.length === 0) {
    return <></>;
  }

  return (
    <div className="card border-0 shadow-sm text-black p-2">
      <div className="card-body d-flex">
        {results.users.map((user, i) => (
          <Link to={`/users/${user.user_uuid}`} key={i}>
            <ProfilePicture
              height="5rem"
              width="5rem"
              className="me-2"
              src={user.picture_url}
            />
          </Link>
        ))}
      </div>
    </div>
  );
}

interface CrateSearchResponse {
  crates: CrateSearchResponseCrate[];
}

interface CrateSearchResponseCrate {
  organisation: string;
  name: string;
  description: string;
  homepage?: string;
  repository?: string;
  version: string;
}

function CrateResults({
  query,
  className,
}: {
  query: string;
  className?: string;
}) {
  const auth = useAuth();

  if (!auth) {
    return <></>;
  }

  const { response: results, error } =
    useAuthenticatedRequest<CrateSearchResponse>(
      {
        auth,
        endpoint: "crates/search?q=" + encodeURIComponent(query),
      },
      [query]
    );

  if (error) {
    return <div className="alert alert-danger">{error}</div>;
  }

  if (!results) {
    return (
      <div className={`card border-0 shadow-sm text-black p-2 ${className}`}>
        <div className="card-body">
          <LoadingSpinner />
        </div>
      </div>
    );
  }

  if (results?.crates.length === 0) {
    return <></>;
  }

  return (
    <div className={`card border-0 shadow-sm text-black ${className}`}>
      <div className="table-responsive">
        <table className="table table-striped">
          <tbody>
            {results?.crates.map((crate, i) => (
              <tr key={i}>
                <td className="p-3">
                  <div className="d-flex flex-row align-items-center">
                    <div
                      className="text-white circle bg-primary bg-gradient d-inline rounded-circle d-inline-flex justify-content-center align-items-center"
                      style={{ width: "2rem", height: "2rem" }}
                    >
                      <BoxSeam />
                    </div>
                    <Link
                      to={`/crates/${crate.organisation}/${crate.name}`}
                      className="text-decoration-none"
                    >
                      <h4 className="text-primary d-inline px-2 m-0">
                        <span className="text-muted">{crate.organisation}</span>
                        /{crate.name}
                      </h4>
                    </Link>
                    <h6 className="text-muted m-0 mt-1">{crate.version}</h6>
                  </div>

                  <p className="m-0">{crate.description}</p>

                  {crate.homepage || crate.repository ? (
                    <div className="mt-2 small">
                      {crate.homepage ? (
                        <a
                          href={crate.homepage}
                          className="text-decoration-none me-2"
                          target="_blank"
                        >
                          Homepage
                        </a>
                      ) : (
                        <></>
                      )}
                      {crate.repository ? (
                        <a
                          href={crate.repository}
                          className="text-decoration-none me-2"
                          target="_blank"
                        >
                          Repository
                        </a>
                      ) : (
                        <></>
                      )}
                    </div>
                  ) : (
                    <></>
                  )}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}
