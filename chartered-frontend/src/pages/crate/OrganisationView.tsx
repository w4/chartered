import React = require("react");
import { useState, useEffect } from "react";
import { Link, useParams } from "react-router-dom";

import Nav from "../../sections/Nav";
import { useAuth } from "../../useAuth";
import { useAuthenticatedRequest, authenticatedEndpoint } from "../../util";

import { BoxSeam, Plus, Trash } from "react-bootstrap-icons";
import {
  Button,
  Dropdown,
  Modal,
  NavLink,
  OverlayTrigger,
  Tooltip,
} from "react-bootstrap";
import HumanTime from "react-human-time";
import ErrorPage from "../ErrorPage";
import Loading from "../Loading";
import Members from "./Members";

interface OrganisationDetails {
  possible_permissions?: string[];
  crates: Crate[];
  members: Member[];
}

interface Crate {
  name: string;
  description?: string;
}

interface Member {
  uuid: string;
  username: string;
  permissions?: string[];
}

export default function ShowOrganisation() {
  const tabs = {
    crates: "Crates",
    members: "Members",
  };

  const { organisation } = useParams();
  const auth = useAuth();
  const [activeTab, setActiveTab] = useState(Object.keys(tabs)[0]);

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
  } else if (!organisationDetails) {
    return <Loading />;
  }

  const description = "a collection of things and stuff.";

  return (
    <div className="text-white">
      <Nav />

      <div className="container mt-4 pb-4">
        <div className="row align-items-stretch">
          <div className="col-12 mb-3">
            <div className="card border-0 shadow-sm text-black h-100">
              <div className="card-body">
                <div className="d-flex flex-row align-items-center">
                  <img
                    src="http://placekitten.com/96/96"
                    className="rounded-circle"
                  />

                  <div className="px-2">
                    <h1 className="text-primary my-0">{organisation}</h1>
                    <p className="m-0">{description}</p>
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

              <div className="card-body">
                <div className="d-flex flex-row align-items-center">
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
                </div>
              </div>
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
                    <span className="text-secondary">{organisation}/</span>
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

function ListMembers({
  organisation,
  members,
  possiblePermissions,
  reload,
}: {
  organisation: string;
  members: Member[];
  possiblePermissions?: string[];
  reload: () => any;
}) {
  const auth = useAuth();

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
