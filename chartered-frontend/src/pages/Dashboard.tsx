import React = require("react");

import { Link } from "react-router-dom";
import { useAuth } from "../useAuth";
import Nav from "../sections/Nav";
import { Calendar3, ChevronRight, Download } from "react-bootstrap-icons";
import { useAuthenticatedRequest } from "../util";
import HumanTime from "react-human-time";
import { OverlayTrigger, Tooltip } from "react-bootstrap";

interface RecentlyCreatedResponse {
  crates: RecentlyCreatedResponseVersion[];
}

interface RecentlyCreatedResponseVersion {
  name: string;
  created_at: string;
  organisation: string;
}

interface RecentlyUpdatedResponse {
  versions: RecentlyUpdatedResponseVersion[];
}

interface RecentlyUpdatedResponseVersion {
  name: string;
  version: string;
  organisation: string;
}

interface MostDownloadedResponse {
  crates: MostDownloadedResponseCrate[];
}

interface MostDownloadedResponseCrate {
  name: string;
  downloads: number;
  organisation: string;
}

export default function Dashboard() {
  const auth = useAuth();

  const { response: recentlyCreated, error: recentlyCreatedError } =
    useAuthenticatedRequest<RecentlyCreatedResponse>({
      auth,
      endpoint: "crates/recently-created",
    });

  const { response: recentlyUpdated, error: recentlyUpdatedError } =
    useAuthenticatedRequest<RecentlyUpdatedResponse>({
      auth,
      endpoint: "crates/recently-updated",
    });

  const { response: mostDownloaded, error: mostDownloadedError } =
    useAuthenticatedRequest<MostDownloadedResponse>({
      auth,
      endpoint: "crates/most-downloaded",
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
            <h4>Newly Created</h4>
            {(recentlyCreated?.crates || []).map((v) => (
              <CrateCard
                key={v.name}
                organisation={v.organisation}
                name={v.name}
              >
                <OverlayTrigger
                  overlay={
                    <Tooltip id={`tooltip-${v.name}-date`}>
                      {new Date(v.created_at).toLocaleString()}
                    </Tooltip>
                  }
                >
                  <span>
                    <Calendar3 />{" "}
                    <HumanTime time={new Date(v.created_at).getTime()} />
                  </span>
                </OverlayTrigger>
              </CrateCard>
            ))}
          </div>

          <div className="col-12 col-md-4">
            <h4>Recently Updated</h4>
            {(recentlyUpdated?.versions || []).map((v) => (
              <CrateCard
                key={v.name}
                organisation={v.organisation}
                name={v.name}
              >
                v{v.version}
              </CrateCard>
            ))}
          </div>

          <div className="col-12 col-md-4">
            <h4>Most Downloaded</h4>
            {(mostDownloaded?.crates || []).map((v) => (
              <CrateCard
                key={v.name}
                organisation={v.organisation}
                name={v.name}
              >
                <Download /> {v.downloads.toLocaleString()}
              </CrateCard>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}

function CrateCard({
  name,
  organisation,
  children,
}: React.PropsWithChildren<{ name: string; organisation: string }>) {
  return (
    <Link
      to={`/crates/${organisation}/${name}`}
      className="text-decoration-none"
    >
      <div className="card border-0 mb-2 shadow-sm">
        <div className="card-body text-black d-flex flex-row">
          <div className="flex-grow-1 align-self-center">
            <h6 className="text-primary my-0">
              <span className="text-secondary">{organisation}/</span>
              {name}
            </h6>
            <small className="text-secondary">{children}</small>
          </div>

          <ChevronRight size={16} className="align-self-center" />
        </div>
      </div>
    </Link>
  );
}
