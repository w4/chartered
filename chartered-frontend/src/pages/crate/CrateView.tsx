import React = require("react");

import { useState, useEffect } from "react";

import { useAuth } from "../../useAuth";
import Nav from "../../sections/Nav";
import Loading from "../Loading";
import ErrorPage from "../ErrorPage";
import { Box, HouseDoor, Book, Building } from "react-bootstrap-icons";
import { useParams } from "react-router-dom";
import { useAuthenticatedRequest } from "../../util";

import Prism from "react-syntax-highlighter/dist/cjs/prism";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import Members from "./Members";

type Tab = "readme" | "versions" | "members";

export interface CrateInfo {
  versions: CrateInfoVersion[];
}

export interface CrateInfoVersion {
  vers: string;
  homepage: string | null;
  description: string | null;
  documentation: string | null;
  repository: string | null;
  deps: CrateInfoVersionDependency[];
}

export interface CrateInfoVersionDependency {
  name: string;
  version_req: string;
}

export default function SingleCrate() {
  const auth = useAuth();
  const { crate } = useParams();

  const [currentTab, setCurrentTab] = useState<Tab>("readme");

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
                    <Box />
                  </div>
                  <h1 className="text-primary d-inline px-2">{crate}</h1>
                  <h2 className="text-secondary m-0">{crateVersion.vers}</h2>
                </div>

                <p className="m-0">{crateVersion.description}</p>
              </div>
            </div>
          </div>

          <div className="col-md-6">
            <div className="card border-0 shadow-sm text-black h-100">
              <div className="card-body">
                <HouseDoor />{" "}
                <a href={crateVersion.homepage}>{crateVersion.homepage}</a>
                <br />
                <Book />{" "}
                <a href={crateVersion.documentation}>
                  {crateVersion.documentation}
                </a>
                <br />
                <Building />{" "}
                <a href={crateVersion.repository}>{crateVersion.repository}</a>
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
                    <a
                      className={`nav-link ${
                        currentTab == "readme"
                          ? "bg-primary bg-gradient active"
                          : ""
                      }`}
                      href="#"
                      onClick={() => setCurrentTab("readme")}
                    >
                      Readme
                    </a>
                  </li>
                  <li className="nav-item">
                    <a
                      className={`nav-link ${
                        currentTab == "versions"
                          ? "bg-primary bg-gradient active"
                          : ""
                      }`}
                      href="#"
                      onClick={() => setCurrentTab("versions")}
                    >
                      Versions
                      <span className={`badge rounded-pill bg-danger ms-1`}>
                        {crateInfo.versions.length}
                      </span>
                    </a>
                  </li>
                  <li className="nav-item">
                    <a
                      className={`nav-link ${
                        currentTab == "members"
                          ? "bg-primary bg-gradient active"
                          : ""
                      }`}
                      href="#"
                      onClick={() => setCurrentTab("members")}
                    >
                      Members
                    </a>
                  </li>
                </ul>
              </div>

              <div className="card-body">
                {currentTab == "readme" ? (
                  <ReadMe crateInfo={crateVersion} />
                ) : (
                  <></>
                )}
                {currentTab == "versions" ? <>Versions</> : <></>}
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

function ReadMe(props: { crateInfo: any }) {
  return (
    <ReactMarkdown
      children={props.crateInfo.readme}
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
