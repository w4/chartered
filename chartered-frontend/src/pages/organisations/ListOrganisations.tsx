import React = require("react");
import { useState, useEffect } from "react";
import { Link } from "react-router-dom";

import Nav from "../../sections/Nav";
import { useAuth } from "../../useAuth";
import { useAuthenticatedRequest, authenticatedEndpoint } from "../../util";

import { Plus, Trash } from "react-bootstrap-icons";
import {
  Button,
  Dropdown,
  Modal,
  OverlayTrigger,
  Tooltip,
} from "react-bootstrap";
import HumanTime from "react-human-time";
import ErrorPage from "../ErrorPage";
import Loading from "../Loading";

export default function ListOrganisations() {
  return (
    <div className="text-white">
      <Nav />

      <div className="container mt-4 pb-4">
        <h1>Your Organisations</h1>

        <div className="card border-0 shadow-sm text-black">
          <table className="table table-striped">
            <tbody>
              <tr>
                <td className="align-middle fit">
                  <img
                    src="http://placekitten.com/48/48"
                    className="rounded-circle"
                  />
                </td>

                <td className="align-middle">
                  <Link to="/crates/core">core</Link>
                </td>

                <td className="fit align-middle">
                  <Dropdown>
                    <Dropdown.Toggle variant=""></Dropdown.Toggle>

                    <Dropdown.Menu>
                      <Dropdown.Item href="#/action-1">Members</Dropdown.Item>
                      <Dropdown.Item href="#/action-2">Crates</Dropdown.Item>
                    </Dropdown.Menu>
                  </Dropdown>
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
}
