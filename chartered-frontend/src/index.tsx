import "./dark.sass"; // TODO: lazyload
import "./index.sass";

import "./overscrollColourFixer.ts";

import { useEffect } from "react";
import ReactDOM from "react-dom";
import {
  BrowserRouter as Router,
  Routes,
  Route,
  useLocation,
  Navigate,
} from "react-router-dom";

import { ProvideAuth, HandleOAuthLogin, useAuth } from "./useAuth";

import Login from "./pages/Login";
import Dashboard from "./pages/Dashboard";
import CrateView from "./pages/crate/CrateView";
import ListSshKeys from "./pages/ssh-keys/ListSshKeys";
import AddSshKeys from "./pages/ssh-keys/AddSshKeys";
import ListOrganisations from "./pages/organisations/ListOrganisations";
import OrganisationView from "./pages/crate/OrganisationView";
import CreateOrganisation from "./pages/organisations/CreateOrganisation";
import User from "./pages/User";
import Search from "./pages/Search";
import { backgroundFix } from "./overscrollColourFixer";
import Register from "./pages/Register";
import ListSessions from "./pages/sessions/ListSessions";
import NotFound from "./pages/NotFound";

if (
  window.matchMedia &&
  window.matchMedia("(prefers-color-scheme: dark)").matches
) {
  document.querySelector("body")?.classList.add("dark");
}

window
  .matchMedia("(prefers-color-scheme: dark)")
  .addEventListener("change", (e) => {
    const body = document.querySelector("body");
    body?.classList.toggle("dark", e.matches);
    backgroundFix(body);
  });

// prettier-ignore
function App() {
  return (
    <ProvideAuth>
      <Router>
        <Routes>
          {/* Public routes, visible only to unauthenticated users */}
          <Route path="/login" element={<Public element={<Login />} />} />
          <Route path="/register" element={<Public element={<Register />} />} />
          <Route path="/login/oauth" element={<Public element={<HandleOAuthLogin />} />} />

          {/* Private routes, visible only to authenticated users */}
          <Route path="/" element={<Private element={<Navigate replace to="/dashboard" />} />} />
          <Route path="/dashboard" element={<Private element={<Dashboard />} />} />
          <Route path="/search" element={<Private element={<Search />} />} />

          <Route path="/crates/:organisation" element={<Private element={<OrganisationView />} />} />
          <Route path="/crates/:organisation/:crate" element={<Private element={<CrateView />} />} />
          <Route path="/crates/:organisation/:crate/:subview" element={<Private element={<CrateView />} />} />

          <Route path="/users/:uuid" element={<Private element={<User />} />} />

          <Route path="/ssh-keys" element={<Private element={<Navigate replace to="/ssh-keys/list" />} />} />
          <Route path="/ssh-keys/list" element={<Private element={<ListSshKeys />} />} />
          <Route path="/ssh-keys/add" element={<Private element={<AddSshKeys />} />} />

          <Route path="/organisations" element={<Private element={<Navigate replace to="/organisations/list" />} />} />
          <Route path="/organisations/list" element={<Private element={<ListOrganisations />} />} />
          <Route path="/organisations/create" element={<Private element={<CreateOrganisation />} />} />

          <Route path="/sessions" element={<Private element={<Navigate replace to="/sessions/list" />} />} />
          <Route path="/sessions/list" element={<Private element={<ListSessions />} />} />

          <Route path="/404" element={<NotFound />} />
          <Route path="*" element={<Navigate replace to="/404" />} />
        </Routes>
      </Router>
    </ProvideAuth>
  );
}

ReactDOM.render(<App />, document.getElementById("root"));

function Public(i: { element: JSX.Element }) {
  const auth = useAuth();
  const location = useLocation();

  if (auth?.getAuthKey()) {
    return (
      <Navigate
        to={location.state?.from?.pathname ?? "/dashboard"}
        state={{ from: location }}
      />
    );
  } else {
    return <>{i.element}</>;
  }
}

function Private(i: { element: JSX.Element }) {
  const auth = useAuth();
  const location = useLocation();

  const isAuthenticated = auth?.getAuthKey();
  useEffect(() => {
    if (!isAuthenticated) {
      auth?.logout();
    }
  }, [isAuthenticated]);

  if (auth && isAuthenticated) {
    return <>{i.element}</>;
  } else {
    return <Navigate to="/login" state={{ from: location }} />;
  }
}
