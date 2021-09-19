import React = require("react");

import { useState, useEffect } from "react";

import { useAuth } from "../../useAuth";
import Nav from "../../sections/Nav";
import Loading from "../Loading";
import ErrorPage from "../ErrorPage";
import {
  BoxSeam,
  HouseDoor,
  Book,
  Building,
  Calendar3,
  Check2Square,
  Hdd,
  CheckSquare,
  Check,
  Square,
} from "react-bootstrap-icons";
import { useParams, NavLink, Redirect } from "react-router-dom";
import { useAuthenticatedRequest } from "../../util";

import Prism from "react-syntax-highlighter/dist/cjs/prism";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import Members from "./Members";
import { OverlayTrigger, Tooltip } from "react-bootstrap";
import HumanTime from "react-human-time";

type Tab = "readme" | "versions" | "members";

export interface CrateInfo {
  name: string;
  readme?: string;
  description?: string;
  repository?: string;
  homepage?: string;
  documentation?: string;
  versions: CrateInfoVersion[];
}

export interface CrateInfoVersion {
  vers: string;
  deps: CrateInfoVersionDependency[];
  features: any[];
  size: number;
  uploader: string;
  created_at: string;
}

export interface CrateInfoVersionDependency {
  name: string;
  version_req: string;
}

export default function SingleCrate() {
  const auth = useAuth();
  const { crate, subview } = useParams();
  const currentTab: Tab | undefined = subview;

  if (!currentTab) {
    return <Redirect to={`/crates/${crate}/readme`} />
  }

  const { response: crateInfo, error } = useAuthenticatedRequest<CrateInfo>({
    auth,
    endpoint: `crates/${crate}`,
  });

  if (error) {
    return <ErrorPage message={error} />;
  } else if (!crateInfo) {
    return <Loading />;
  }

  const crateVersion = crateInfo.versions[crateInfo.versions.length - 1];

  return (
    <div className="text-white">
      <Nav />

      <div className="container mt-4 pb-4">
        <div className="row align-items-stretch">
          <div className="col-md-6">
            <div className="card border-0 shadow-sm text-black h-100">
              <div className="card-body">
                <div className="d-flex flex-row align-items-center">
                  <div
                    className="text-white circle bg-primary bg-gradient d-inline rounded-circle d-inline-flex justify-content-center align-items-center"
                    style={{ width: "2rem", height: "2rem" }}
                  >
                    <BoxSeam />
                  </div>
                  <h1 className="text-primary d-inline px-2">{crate}</h1>
                  <h2 className="text-secondary m-0">{crateVersion.vers}</h2>
                </div>

                <p className="m-0">{crateInfo.description}</p>
              </div>
            </div>
          </div>

          <div className="col-md-6">
            <div className="card border-0 shadow-sm text-black h-100">
              <div className="card-body d-flex flex-column justify-content-center">
                {crateInfo.homepage ? (
                  <div>
                    <HouseDoor />{" "}
                    <a href={crateInfo.homepage}>{crateInfo.homepage}</a>
                  </div>
                ) : (
                  <></>
                )}
                {crateInfo.documentation ? (
                  <div>
                    <Book />{" "}
                    <a href={crateInfo.documentation}>
                      {crateInfo.documentation}
                    </a>
                  </div>
                ) : (
                  <></>
                )}
                {crateInfo.repository ? (
                  <div>
                    <Building />{" "}
                    <a href={crateInfo.repository}>{crateInfo.repository}</a>
                  </div>
                ) : (
                  <></>
                )}
              </div>
            </div>
          </div>
        </div>

        <div className="row my-4">
          <div className="col-md-9">
            <div className="card border-0 shadow-sm text-black">
              <div className="card-header">
                <ul className="nav nav-pills card-header-pills">
                  <li className="nav-item">
                    <NavLink to={`/crates/${crate}/readme`} className="nav-link" activeClassName="bg-primary bg-gradient active">
                      Readme
                    </NavLink>
                  </li>
                  <li className="nav-item">
                    <NavLink to={`/crates/${crate}/versions`} className="nav-link" activeClassName="bg-primary bg-gradient active">
                      Versions
                      <span className={`badge rounded-pill bg-danger ms-1`}>
                        {crateInfo.versions.length}
                      </span>
                    </NavLink>
                  </li>
                  <li className="nav-item">
                    <NavLink to={`/crates/${crate}/members`} className="nav-link" activeClassName="bg-primary bg-gradient active">
                      Members
                    </NavLink>
                  </li>
                </ul>
              </div>

              <div className="card-body">
                {currentTab == "readme" ? <ReadMe crate={crateInfo} /> : <></>}
                {currentTab == "versions" ? (
                  <Versions crate={crateInfo} />
                ) : (
                  <></>
                )}
                {currentTab == "members" ? <Members crate={crate} /> : <></>}
              </div>
            </div>
          </div>

          <div className="col-md-3">
            <div className="card border-0 shadow-sm text-black">
              <div className="card-body pb-0">
                <h5 className="card-title">Dependencies</h5>
              </div>

              <ul className="list-group list-group-flush mb-2">
                {crateVersion.deps.map((dep) => (
                  <li
                    key={`${dep.name}-${dep.version_req}`}
                    className="list-group-item"
                  >
                    {dep.name} = "<strong>{dep.version_req}</strong>"
                  </li>
                ))}
              </ul>
            </div>

            <div className="card border-0 shadow-sm text-black mt-4">
              <div className="card-body pb-0">
                <h5 className="card-title">Dependents</h5>
              </div>

              <ul className="list-group list-group-flush">
                <li className="list-group-item">An item</li>
                <li className="list-group-item">A second item</li>
                <li className="list-group-item">A third item</li>
              </ul>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

function Versions(props: { crate: CrateInfo }) {
  const humanFileSize = (size) => {
    var i = Math.floor(Math.log(size) / Math.log(1024));
    return (
      Number((size / Math.pow(1024, i)).toFixed(2)) +
      " " +
      ["B", "kB", "MB", "GB", "TB"][i]
    );
  };

  return (
    <div>
      {[...props.crate.versions].reverse().map((version, index) => (
        <div
          key={index}
          className={`card text-white bg-gradient ${
            index == 0 ? "bg-primary" : "bg-dark mt-2"
          }`}
        >
          <div className="card-body d-flex align-items-center">
            <h5 className="m-0">{version.vers}</h5>

            <div className="text-uppercase ms-4" style={{ fontSize: ".75rem" }}>
              <div>
                <div className="d-inline-block">
                  By
                  <img
                    src="http://placekitten.com/22/22"
                    className="rounded-circle ms-1 me-1"
                  />
                  {version.uploader}
                </div>

                <div className="ms-3 d-inline-block">
                  <OverlayTrigger
                    overlay={
                      <Tooltip
                        id={`tooltip-${props.crate.name}-version-${version.vers}-date`}
                      >
                        {new Date(version.created_at).toLocaleString()}
                      </Tooltip>
                    }
                  >
                    <span>
                      <Calendar3 />{" "}
                      <HumanTime
                        time={new Date(version.created_at).getTime()}
                      />
                    </span>
                  </OverlayTrigger>
                </div>
              </div>

              <div>
                <div className="d-inline-block">
                  <Hdd /> {humanFileSize(version.size)}
                </div>

                <div className="ms-3 d-inline-block">
                  <OverlayTrigger
                    overlay={
                      <Tooltip
                        id={`tooltip-${props.crate.name}-version-${version.vers}-feature-${index}`}
                      >
                        <div className="text-start m-2">
                          {Object.keys(version.features).map(
                            (feature, index) => (
                              <div key={index}>
                                {version.features["default"].includes(
                                  feature
                                ) ? (
                                  <CheckSquare className="me-2" />
                                ) : (
                                  <Square className="me-2" />
                                )}
                                {feature}
                              </div>
                            )
                          )}
                        </div>
                      </Tooltip>
                    }
                  >
                    <span>
                      <Check2Square /> {Object.keys(version.features).length}{" "}
                      Features
                    </span>
                  </OverlayTrigger>
                </div>
              </div>
            </div>
          </div>
        </div>
      ))}
    </div>
  );
}

function ReadMe(props: { crate: CrateInfo }) {
  return (
    <ReactMarkdown
      children={props.crate.readme}
      remarkPlugins={[remarkGfm]}
      components={{
        code({ node, inline, className, children, ...props }) {
          const match = /language-(\w+)/.exec(className || "");
          return !inline && match ? (
            <Prism
              children={String(children).replace(/\n$/, "")}
              language={match[1]}
              PreTag="pre"
              {...props}
            />
          ) : (
            <code className={className} {...props}>
              {children}
            </code>
          );
        },
      }}
    />
  );
}
