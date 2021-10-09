import "./index.sass";

import "./overscrollColourFixer.ts";

import React = require("react");
import ReactDOM = require("react-dom");
import {
  BrowserRouter as Router,
  Switch,
  Route,
  Redirect,
  useLocation,
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

function App() {
  return (
    <ProvideAuth>
      <Router>
        <Switch>
          <PublicRoute
            exact
            unauthedOnly
            path="/login"
            component={() => <Login />}
          />
          <PublicRoute
            exact
            unauthedOnly
            path="/login/oauth"
            component={() => <HandleOAuthLogin />}
          />

          <PrivateRoute
            exact
            path="/"
            component={() => <Redirect to="/dashboard" />}
          />
          <PrivateRoute
            exact
            path="/dashboard"
            component={() => <Dashboard />}
          />
          <PrivateRoute
            exact
            path="/crates/:organisation"
            component={() => <OrganisationView />}
          />
          <PrivateRoute
            exact
            path="/crates/:organisation/:crate/:subview?"
            component={() => <CrateView />}
          />
          <PrivateRoute exact path="/users/:uuid" component={() => <User />} />
          <PrivateRoute
            exact
            path="/ssh-keys"
            component={() => <Redirect to="/ssh-keys/list" />}
          />
          <PrivateRoute
            exact
            path="/ssh-keys/list"
            component={() => <ListSshKeys />}
          />
          <PrivateRoute
            exact
            path="/ssh-keys/add"
            component={() => <AddSshKeys />}
          />
          <PrivateRoute
            exact
            path="/organisations"
            component={() => <Redirect to="/organisations/list" />}
          />
          <PrivateRoute
            exact
            path="/organisations/list"
            component={() => <ListOrganisations />}
          />
          <PrivateRoute
            exact
            path="/organisations/create"
            component={() => <CreateOrganisation />}
          />
          <PrivateRoute
            exact
            path="/search"
            component={() => <Search />}
          />
        </Switch>
      </Router>
    </ProvideAuth>
  );
}

ReactDOM.render(<App />, document.getElementById("root"));

function PublicRoute({
  component: Component,
  unauthedOnly,
  ...rest
}: {
  component: ({
    match,
    location,
  }: {
    match: any;
    location: ReturnType<typeof useLocation>;
  }) => JSX.Element;
  unauthedOnly: boolean;
} & { [r: string]: any }) {
  const auth = useAuth();

  return (
    <Route
      {...rest}
      render={(props) => {
        // TODO: check if valid key
        if (!unauthedOnly || !auth || !auth?.getAuthKey()) {
          return <Component {...props} />;
        } else {
          return (
            <Redirect
              to={{
                pathname: props.location.state?.from?.pathname ?? "/dashboard",
                state: { from: props.location },
              }}
            />
          );
        }
      }}
    ></Route>
  );
}

function PrivateRoute({
  component: Component,
  ...rest
}: {
  component: ({
    match,
    location,
  }: {
    match: any;
    location: ReturnType<typeof useLocation>;
  }) => JSX.Element;
} & { [r: string]: any }) {
  const auth = useAuth();

  const isAuthenticated = auth?.getAuthKey();
  React.useEffect(() => {
    if (!isAuthenticated) {
      auth.logout();
    }
  }, [isAuthenticated]);

  return (
    <Route
      {...rest}
      render={(props) => {
        // TODO: check if valid key
        if (auth && isAuthenticated) {
          return <Component {...props} />;
        } else {
          return (
            <Redirect
              to={{ pathname: "/login", state: { from: props.location } }}
            />
          );
        }
      }}
    ></Route>
  );
}
