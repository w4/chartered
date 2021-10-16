import { Plus } from "react-bootstrap-icons";
import {Link, Redirect} from "react-router-dom";

import Nav from "../../sections/Nav";
import { useAuth } from "../../useAuth";
import { RoundedPicture, useAuthenticatedRequest } from "../../util";
import ErrorPage from "../ErrorPage";
import { LoadingSpinner } from "../Loading";

interface Response {
  organisations: ResponseOrganisations[];
}

interface ResponseOrganisations {
  name: string;
  description: string;
}

export default function ListOrganisations() {
  const auth = useAuth();

  if (!auth) {
    return <Redirect to="/login" />;
  }

  const { response: list, error } = useAuthenticatedRequest<Response>({
    auth,
    endpoint: "organisations",
  });

  if (error) {
    return <ErrorPage message={error} />;
  }

  return (
    <div>
      <Nav />

      <div className="container mt-4 pb-4">
        <h1>Your Organisations</h1>

        <div className="card border-0 shadow-sm text-black">
          {!list ? (
            <LoadingSpinner />
          ) : (
            <>
              {list.organisations.length === 0 ? (
                <div className="card-body">
                  You don't belong to any organisations yet.
                </div>
              ) : (
                <table className="table table-striped">
                  <tbody>
                    {list.organisations.map((v, i) => (
                      <tr key={i}>
                        <td className="align-middle fit">
                          <RoundedPicture
                            src="http://placekitten.com/48/48"
                            height="48px"
                            width="48px"
                          />
                        </td>

                        <td
                          className="align-middle"
                          style={{ lineHeight: "1.1" }}
                        >
                          <div>
                            <Link to={`/crates/${v.name}`}>{v.name}</Link>
                          </div>
                          <div>
                            <small style={{ fontSize: "0.75rem" }}>
                              {v.description}
                            </small>
                          </div>
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              )}
            </>
          )}
        </div>

        <Link
          to="/organisations/create"
          className="btn btn-outline-light mt-2 float-end"
        >
          <Plus /> Create
        </Link>
      </div>
    </div>
  );
}
