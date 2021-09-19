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

import { ProvideAuth, useAuth } from "./useAuth";

import Login from "./pages/Login";
import Dashboard from "./pages/Dashboard";
import CrateView from "./pages/crate/CrateView";
import ListSshKeys from "./pages/ssh-keys/ListSshKeys";
import AddSshKeys from "./pages/ssh-keys/AddSshKeys";

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
            path="/crates/:crate/:subview?"
            component={() => <CrateView />}
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
        if (
          !unauthedOnly ||
          !auth ||
          !auth.authKey ||
          auth.expires < new Date()
        ) {
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

  return (
    <Route
      {...rest}
      render={(props) => {
        // TODO: check if valid key
        if (auth && auth?.authKey && auth.expires > new Date()) {
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
