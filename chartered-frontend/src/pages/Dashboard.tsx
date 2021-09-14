import React = require('react');

import { useAuth } from '../useAuth';
import { ChevronRight } from 'react-bootstrap-icons';

export default function Dashboard() {
    const auth = useAuth();

    return (
        <div className="text-white">
            <nav className="navbar navbar-expand-lg navbar-light bg-white shadow-sm">
                <div className="container-fluid">
                    <a className="navbar-brand" href="#">✈️ chartered</a>
                    <button className="navbar-toggler" type="button" data-bs-toggle="collapse" data-bs-target="#navbarSupportedContent" aria-controls="navbarSupportedContent" aria-expanded="false" aria-label="Toggle navigation">
                        <span className="navbar-toggler-icon"></span>
                    </button>

                    <div className="collapse navbar-collapse" id="navbarSupportedContent">
                        <ul className="navbar-nav me-auto mb-2 mb-lg-0">
                            <li className="nav-item">
                                <a className="nav-link active" aria-current="page" href="#">Home</a>
                            </li>
                            <li className="nav-item">
                                <a className="nav-link" href="#">Link</a>
                            </li>
                            <li className="nav-item dropdown">
                                <a className="nav-link dropdown-toggle" href="#" id="navbarDropdown" role="button" data-bs-toggle="dropdown" aria-expanded="false">
                                    Dropdown
                                </a>

                                <ul className="dropdown-menu" aria-labelledby="navbarDropdown">
                                    <li><a className="dropdown-item" href="#">Action</a></li>
                                    <li><a className="dropdown-item" href="#">Another action</a></li>
                                    <li><hr className="dropdown-divider"></li>
                                    <li><a className="dropdown-item" href="#">Something else here</a></li>
                                </ul>
                            </li>

                            <li className="nav-item">
                                <a className="nav-link disabled" href="#" taIndex="-1" aria-disabled="true">Disabled</a>
                            </li>
                        </ul>

                        <form className="d-flex">
                            <input className="form-control me-2" type="search" placeholder="Search" aria-label="Search" />
                            <button className="btn btn-outline-success" type="submit">Search</button>
                        </form>
                    </div>
                </div>
            </nav>

            <div className="container my-4">
                <h1 className="mb-0">Welcome to Chartered.</h1>
                <p style={{maxWidth: '72ch'}}>
                    A private, authenticated Cargo registry. Everything published to this registry is <em>private and visible only to you</em>,
                    until explicit permissions are granted to others.
                </p>
                <div className="btn btn-light btn-lg shadow-sm">Getting Started</div>

                <hr />

                <div className="row">
                    <div className="col-md-4">
                        <h4>Your Crates</h4>

                        <div className="card border-0 mb-2 shadow-sm">
                            <div className="card-body text-black d-flex flex-row">
                                <div className="flex-grow-1 align-self-center">
                                    <h6 className="text-primary my-0">hello-world-rs <small>(owner)</small></h6>
                                    <small className="text-secondary">v0.0.1</small>
                                </div>

                                <ChevronRight size={16} className="align-self-center" />
                            </div>
                        </div>

                        <div className="card border-0 mb-2 shadow-sm">
                            <div className="card-body text-black d-flex flex-row">
                                <div className="flex-grow-1 align-self-center">
                                    <h6 className="text-primary my-0">cool-beans-rs <small>(contributor)</small></h6>
                                    <small className="text-secondary">v0.0.1</small>
                                </div>

                                <ChevronRight size={16} className="align-self-center" />
                            </div>
                        </div>
                    </div>

                    <div className="col-md-4">
                        <h4>Recently Updated</h4>

                        <div className="card border-0 mb-2 shadow-sm">
                            <div className="card-body text-black d-flex flex-row">
                                <div className="flex-grow-1 align-self-center">
                                    <h6 className="text-primary my-0">hello-world-rs <small>(owner)</small></h6>
                                    <small className="text-secondary">v0.0.1</small>
                                </div>

                                <ChevronRight size={16} className="align-self-center" />
                            </div>
                        </div>

                        <div className="card border-0 mb-2 shadow-sm">
                            <div className="card-body text-black d-flex flex-row">
                                <div className="flex-grow-1 align-self-center">
                                    <h6 className="text-primary my-0">cool-beans-rs <small>(contributor)</small></h6>
                                    <small className="text-secondary">v0.0.1</small>
                                </div>

                                <ChevronRight size={16} className="align-self-center" />
                            </div>
                        </div>
                    </div>

                    <div className="col-md-4">
                        <h4>Most Downloaded</h4>

                        <div className="card border-0 mb-2 shadow-sm">
                            <div className="card-body text-black d-flex flex-row">
                                <div className="flex-grow-1 align-self-center">
                                    <h6 className="text-primary my-0">hello-world-rs <small>(owner)</small></h6>
                                    <small className="text-secondary">v0.0.1</small>
                                </div>

                                <ChevronRight size={16} className="align-self-center" />
                            </div>
                        </div>

                        <div className="card border-0 mb-2 shadow-sm">
                            <div className="card-body text-black d-flex flex-row">
                                <div className="flex-grow-1 align-self-center">
                                    <h6 className="text-primary my-0">cool-beans-rs <small>(contributor)</small></h6>
                                    <small className="text-secondary">v0.0.1</small>
                                </div>

                                <ChevronRight size={16} className="align-self-center" />
                            </div>
                        </div>
                    </div>
                </div>

                <a onClick={() => auth.logout()}>logout</a>
            </div>
        </div>
    );
}
