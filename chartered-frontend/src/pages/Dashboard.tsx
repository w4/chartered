import React = require("react");

import { Link } from "react-router-dom";
import { useAuth } from "../useAuth";
import Nav from "../sections/Nav";
import { ChevronRight } from "react-bootstrap-icons";
import { useAuthenticatedRequest } from "../util";

interface RecentlyUpdatedResponse {
  versions: RecentlyUpdatedResponseVersion[];
}

interface RecentlyUpdatedResponseVersion {
  name: string;
  version: string;
  organisation: string;
}

export default function Dashboard() {
  const auth = useAuth();

  const { response: recentlyUpdated, error } =
    useAuthenticatedRequest<RecentlyUpdatedResponse>({
      auth,
      endpoint: "crates/recently-updated",
    });

  return (
    <div className="text-white">
      <Nav />

      <div className="container mt-4 pb-4">
        <h1 className="mb-0">Welcome to Chartered.</h1>
        <p style={{ maxWidth: "72ch" }}>
          A private, authenticated Cargo registry. Everything published to this
          registry is <em>private and visible only to you</em>, until explicit
          permissions are granted to others.
        </p>
        <a
          href="https://book.chart.rs/"
          target="_blank"
          className="btn btn-outline-light shadow-sm"
        >
          Getting Started
        </a>

        <hr />

        <div className="row">
          <div className="col-12 col-md-4">
            <h4>Your Crates</h4>
          </div>

          <div className="col-12 col-md-4">
            <h4>Recently Updated</h4>
            {(recentlyUpdated?.versions || []).map((v) => (
              <CrateCard key={v.name} crate={v} />
            ))}
          </div>

          <div className="col-12 col-md-4">
            <h4>Most Downloaded</h4>
          </div>
        </div>
      </div>
    </div>
  );
}

interface Crate {
  name: string;
  version: string;
  organisation: string;
}

function CrateCard({ crate }: { crate: Crate }) {
  return (
    <Link
      to={`/crates/${crate.organisation}/${crate.name}`}
      className="text-decoration-none"
    >
      <div className="card border-0 mb-2 shadow-sm">
        <div className="card-body text-black d-flex flex-row">
          <div className="flex-grow-1 align-self-center">
            <h6 className="text-primary my-0">
              <span className="text-secondary">{crate.organisation}/</span>
              {crate.name}
            </h6>
            <small className="text-secondary">v{crate.version}</small>
          </div>

          <ChevronRight size={16} className="align-self-center" />
        </div>
      </div>
    </Link>
  );
}
