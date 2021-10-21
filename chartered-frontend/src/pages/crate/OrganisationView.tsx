import { useState } from "react";
import { Link, Redirect, useParams } from "react-router-dom";

import Nav from "../../sections/Nav";
import { useAuth } from "../../useAuth";
import {
  useAuthenticatedRequest,
  authenticatedEndpoint,
  RoundedPicture,
} from "../../util";

import { BoxSeam } from "react-bootstrap-icons";
import ErrorPage from "../ErrorPage";
import Members from "./Members";
import ReactPlaceholder from "react-placeholder";

interface OrganisationDetails {
  possible_permissions?: string[];
  crates: Crate[];
  members: Member[];
  description: string;
}

interface Crate {
  name: string;
  description?: string;
}

interface Member {
  uuid: string;
  display_name: string;
  permissions: string[];
}

interface UrlParams {
  organisation: string;
}

export default function ShowOrganisation() {
  const tabs = {
    crates: "Crates",
    members: "Members",
  };

  const { organisation } = useParams<UrlParams>();
  const auth = useAuth();
  const [activeTab, setActiveTab] = useState(Object.keys(tabs)[0]);

  if (!auth) {
    return <Redirect to="/login" />;
  }

  const [reload, setReload] = useState(0);
  const { response: organisationDetails, error } =
    useAuthenticatedRequest<OrganisationDetails>(
      {
        auth,
        endpoint: `organisations/${organisation}`,
      },
      [reload]
    );

  if (error) {
    return <ErrorPage message={error} />;
  }

  const ready = !!organisationDetails;

  return (
    <div>
      <Nav />

      <div className="container mt-4 pb-4">
        <div className="row align-items-stretch">
          <div className="col-12 mb-3">
            <div className="card border-0 shadow-sm text-black h-100">
              <div className="card-body">
                <div className="d-flex flex-row align-items-center">
                  <RoundedPicture
                    src="http://placekitten.com/96/96"
                    height="96px"
                    width="96px"
                  />

                  <div className="px-2">
                    <h1 className="text-primary my-0">{organisation}</h1>
                    <ReactPlaceholder
                      showLoadingAnimation
                      type="text"
                      rows={1}
                      ready={ready}
                      style={{ height: "1.4rem" }}
                    >
                      <p className="m-0">
                        {organisationDetails?.description || (
                          <i>No description given.</i>
                        )}
                      </p>
                    </ReactPlaceholder>
                  </div>
                </div>
              </div>
            </div>
          </div>

          <div className="col-12 mb-3">
            <div className="card border-0 shadow-sm text-black h-100">
              <div className="card-header">
                <ul className="nav nav-pills card-header-pills">
                  {Object.entries(tabs).map(([key, name]) => (
                    <li key={key} className="nav-item">
                      <a
                        href="#"
                        className={`nav-link ${
                          activeTab == key
                            ? "bg-primary bg-gradient active"
                            : ""
                        }`}
                        onClick={(e) => {
                          e.preventDefault();
                          setActiveTab(key);
                        }}
                      >
                        {name}
                      </a>
                    </li>
                  ))}
                </ul>
              </div>

              {!ready ? (
                <div className="card-body">
                  <div className="d-flex justify-content-center align-items-center">
                    <div className="spinner-border text-primary" role="status">
                      <span className="visually-hidden">Loading...</span>
                    </div>
                  </div>
                </div>
              ) : (
                <>
                  {activeTab == "crates" ? (
                    <ListCrates
                      organisation={organisation}
                      crates={organisationDetails.crates}
                    />
                  ) : (
                    <></>
                  )}
                  {activeTab == "members" ? (
                    <ListMembers
                      organisation={organisation}
                      members={organisationDetails.members}
                      possiblePermissions={
                        organisationDetails.possible_permissions
                      }
                      reload={() => setReload(reload + 1)}
                    />
                  ) : (
                    <></>
                  )}
                </>
              )}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

function ListCrates({
  organisation,
  crates,
}: {
  organisation: string;
  crates: Crate[];
}) {
  if (crates.length === 0) {
    return (
      <div className="card-body">
        This organisation doesn't have any crates yet.
      </div>
    );
  }

  return (
    <div className="table-responsive w-100">
      <table className="table table-striped">
        <tbody>
          {crates.map((v, i) => (
            <tr key={i}>
              <td className="align-middle fit">
                <div
                  className="text-white circle bg-primary bg-gradient d-inline rounded-circle d-inline-flex justify-content-center align-items-center"
                  style={{ width: "48px", height: "48px" }}
                >
                  <BoxSeam />
                </div>
              </td>

              <td className="align-middle">
                <div>
                  <Link to={`/crates/${organisation}/${v.name}`}>
                    <span className="text-muted">{organisation}/</span>
                    {v.name}
                  </Link>
                </div>
                <div className="text-muted">{v.description}</div>
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

interface ListMemberParams {
  organisation: string;
  members: Member[];
  possiblePermissions?: string[];
  reload: () => any;
}

function ListMembers({
  organisation,
  members,
  possiblePermissions,
  reload,
}: ListMemberParams) {
  const auth = useAuth();

  if (!auth) {
    return <></>;
  }

  const saveMemberPermissions = async (
    prospectiveMember: boolean,
    uuid: string,
    selectedPermissions: string[]
  ) => {
    let res = await fetch(
      authenticatedEndpoint(auth, `organisations/${organisation}/members`),
      {
        method: prospectiveMember ? "PUT" : "PATCH",
        headers: {
          Accept: "application/json",
          "Content-Type": "application/json",
        },
        body: JSON.stringify({
          user_uuid: uuid,
          permissions: selectedPermissions,
        }),
      }
    );
    let json = await res.json();

    if (json.error) {
      throw new Error(json.error);
    }

    reload();
  };

  const deleteMember = async (uuid: string) => {
    let res = await fetch(
      authenticatedEndpoint(auth, `organisations/${organisation}/members`),
      {
        method: "DELETE",
        headers: {
          Accept: "application/json",
          "Content-Type": "application/json",
        },
        body: JSON.stringify({
          user_uuid: uuid,
        }),
      }
    );
    let json = await res.json();

    if (json.error) {
      throw new Error(json.error);
    }

    reload();
  };

  return (
    <Members
      members={members}
      possiblePermissions={possiblePermissions}
      saveMemberPermissions={saveMemberPermissions}
      deleteMember={deleteMember}
    />
  );
}
