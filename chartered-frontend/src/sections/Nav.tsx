import {SyntheticEvent, useState} from "react";
import { useHistory, useLocation } from "react-router-dom";
import { NavLink, Link } from "react-router-dom";

import { BoxArrowRight, Search } from "react-bootstrap-icons";
import { useAuth } from "../useAuth";
import { Dropdown, Navbar } from "react-bootstrap";
import { ProfilePicture } from "../util";

export default function Nav() {
  const auth = useAuth();
  const history = useHistory();
  const location = useLocation();

  const logout = async (e: SyntheticEvent) => {
    e.preventDefault();
    await auth?.logout();
  };

  const [search, setSearch] = useState(
    location.pathname === "/search"
      ? new URLSearchParams(location.search).get("q") || ""
      : ""
  );
  const submitSearchForm = (e: SyntheticEvent) => {
    e.preventDefault();

    if (search != "") {
      history.push(`/search?q=${encodeURIComponent(search)}`);
    }
  };

  return (
    <Navbar bg="light" expand="md" className="bg-white shadow-sm">
      <div className="container-fluid">
        <Link className="navbar-brand" to="/dashboard">
          ✈️ chartered
        </Link>
        <Navbar.Toggle aria-controls="navbar-contents" />

        <Navbar.Collapse id="navbar-contents" role="navigation">
          <ul className="navbar-nav mb-2 mb-md-0 me-auto">
            <li className="nav-item">
              <NavLink to="/dashboard" className="nav-link">
                Home
              </NavLink>
            </li>
            <li className="nav-item">
              <NavLink to="/ssh-keys" className="nav-link">
                SSH Keys
              </NavLink>
            </li>
            <li className="nav-item">
              <NavLink to="/organisations" className="nav-link">
                Organisations
              </NavLink>
            </li>
          </ul>

          <form className="d-flex" onSubmit={submitSearchForm}>
            <div className="input-group">
              <span className="input-group-text bg-transparent border-none">
                <Search />
              </span>

              <input
                className="form-control me-2"
                type="search"
                placeholder="Search"
                aria-label="Search"
                value={search}
                onChange={(e) => setSearch(e.target.value)}
              />
            </div>
          </form>

          <ul className="navbar-nav">
            <li className="nav-item">
              <Dropdown as="div" className="mt-2 mt-md-0">
                <Dropdown.Toggle
                  as="a"
                  role="button"
                  aria-label="View profile and more"
                  style={{ color: "rgba(0, 0, 0, 0.55)" }}
                  className="d-inline-flex align-items-center"
                >
                  <ProfilePicture
                    src={auth?.getPictureUrl()}
                    height="2rem"
                    width="2rem"
                  />
                </Dropdown.Toggle>

                <Dropdown.Menu
                  align={{ md: "end" }}
                  style={{ marginTop: "5px" }}
                >
                  <Dropdown.Item as={Link} to={`/users/${auth?.getUserUuid()}`}>
                    Your profile
                  </Dropdown.Item>

                  <Dropdown.Divider />

                  <Dropdown.Item
                    as="a"
                    href="#"
                    onClick={logout}
                    className="text-danger"
                  >
                    Logout <BoxArrowRight />
                  </Dropdown.Item>
                </Dropdown.Menu>
              </Dropdown>
            </li>
          </ul>
        </Navbar.Collapse>
      </div>
    </Navbar>
  );
}
