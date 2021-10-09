import React = require("react");
import { useState, useEffect } from "react";
import { Link, useHistory, useLocation } from "react-router-dom";

import Nav from "../sections/Nav";
import { useAuth } from "../useAuth";
import { authenticatedEndpoint, ProfilePicture, useAuthenticatedRequest } from "../util";

import { Plus } from "react-bootstrap-icons";
import { LoadingSpinner } from "./Loading";

interface UsersSearchResponse {
    users: UserSearchResponseUser[];
}

interface UserSearchResponseUser {
    user_uuid: string;
    display_name: string;
    picture_url: string;
}

export default function Search() {
  const auth = useAuth();
  const location = useLocation();

  const query = location.pathname === '/search'
    ? new URLSearchParams(location.search).get("q") || ""
    : "";

  return (
    <div className="text-white">
      <Nav />

      <div className="container mt-4 pb-4">
        <h1>Search Results {query ? <>for '{query}'</> : <></>}</h1>

        <UsersResults query={query} />
      </div>
    </div>
  );
}

function UsersResults({ query }: { query: string }) {
    const auth = useAuth();

    const { response: results, error } =
        useAuthenticatedRequest<UsersSearchResponse>({
            auth,
            endpoint: "users/search?q=" + encodeURIComponent(query),
        }, [query]);

    if (!results) {
        return <div className="card border-0 shadow-sm text-black p-2">
            <div className="card-body">
                {[0, 1, 2].map((i) => (
                    <ProfilePicture key={i} height="5rem" width="5rem" className="me-2" src={undefined} />
                ))}
            </div>
        </div>
    }

    if (results?.users.length === 0) {
        return <></>;
    }

    return <div className="card border-0 shadow-sm text-black p-2">
        <div className="card-body">            
            {results.users.map((user, i) => (
                <Link to={`users/${user.user_uuid}`}>
                    <ProfilePicture key={i} height="5rem" width="5rem" className="me-2" src={user.picture_url} />
                </Link>
            ))}
        </div>
    </div>;
}