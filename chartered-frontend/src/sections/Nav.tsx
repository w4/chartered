import React = require("react");
import { useHistory, useLocation } from "react-router-dom";
import { NavLink, Link } from "react-router-dom";

import { BoxArrowRight, Search } from "react-bootstrap-icons";
import { useAuth } from "../useAuth";

export default function Nav() {
  const auth = useAuth();
  const history = useHistory();
  const location = useLocation();

  const logout = async (e) => {
    e.preventDefault();
    await auth.logout();
  };

  const [search, setSearch] = React.useState(
    location.pathname === "/search"
      ? new URLSearchParams(location.search).get("q") || ""
      : ""
  );
  const submitSearchForm = (e) => {
    e.preventDefault();

    if (search != "") {
      history.push(`/search?q=${encodeURIComponent(search)}`);
    }
  };

  return (
    <nav className="navbar navbar-expand-lg navbar-light bg-white shadow-sm">
      <div className="container-fluid">
        <Link className="navbar-brand" to="/dashboard">
          ✈️ chartered
        </Link>
        <button
          className="navbar-toggler"
          type="button"
          data-bs-toggle="collapse"
          data-bs-target="#navbarSupportedContent"
          aria-controls="navbarSupportedContent"
          aria-expanded="false"
          aria-label="Toggle navigation"
        >
          <span className="navbar-toggler-icon"></span>
        </button>

        <div className="collapse navbar-collapse" id="navbarSupportedContent">
          <ul className="navbar-nav me-auto mb-2 mb-lg-0">
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

          <div>
            <a href="#" onClick={logout} className="nav-link text-danger">
              Logout <BoxArrowRight />
            </a>
          </div>
        </div>
      </div>
    </nav>
  );
}
