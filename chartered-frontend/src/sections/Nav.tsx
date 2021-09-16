import React = require("react");
import { NavLink, Link } from "react-router-dom";

import { BoxArrowRight } from 'react-bootstrap-icons';
import { useAuth } from "../useAuth";

export default function Nav() {
    const auth = useAuth();

    const logout = async (e) => {
        e.preventDefault();
        await auth.logout();
    };

    return (
        <nav className="navbar navbar-expand-lg navbar-light bg-white shadow-sm">
            <div className="container-fluid">
                <Link className="navbar-brand" to="/dashboard">✈️ chartered</Link>
                <button className="navbar-toggler" type="button" data-bs-toggle="collapse" data-bs-target="#navbarSupportedContent" aria-controls="navbarSupportedContent" aria-expanded="false" aria-label="Toggle navigation">
                    <span className="navbar-toggler-icon"></span>
                </button>

                <div className="collapse navbar-collapse" id="navbarSupportedContent">
                    <ul className="navbar-nav me-auto mb-2 mb-lg-0">
                        <li className="nav-item">
                            <NavLink to="/dashboard" className="nav-link">Home</NavLink>
                        </li>
                        <li className="nav-item">
                            <NavLink to="/ssh-keys/list" className="nav-link">SSH Keys</NavLink>
                        </li>
                    </ul>

                    <form className="d-flex">
                        <input className="form-control me-2" type="search" placeholder="Search" aria-label="Search" />
                    </form>

                    <div>
                        <a href="#" onClick={logout} className="nav-link text-danger">Logout <BoxArrowRight /></a>
                    </div>
                </div>
            </div>
        </nav>
    );
}