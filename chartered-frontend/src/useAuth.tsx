import React = require("react");
import { useState, useEffect, useContext, createContext } from "react";
import { useLocation, Redirect } from "react-router-dom";
import { unauthenticatedEndpoint } from "./util";
import LoadingPage from "./pages/Loading";

export interface OAuthProviders {
  providers: string[];
}

export interface AuthContext {
  login: (username: string, password: string) => Promise<void>;
  oauthLogin: (provider: string) => Promise<void>;
  logout: () => Promise<void>;
  getAuthKey: () => Promise<string | null>;
  setAuth: ([string, string]) => any;
}

const authContext = createContext<AuthContext | null>(null);

export function ProvideAuth({ children }: { children: any }) {
  const auth = useProvideAuth();
  return <authContext.Provider value={auth}>{children}</authContext.Provider>;
}

export function HandleOAuthLogin() {
  const location = useLocation();
  const auth = useAuth();
  const [result, setResult] = useState<JSX.Element | null>(null);

  useEffect(async () => {
    try {
      let result = await fetch(unauthenticatedEndpoint(`login/oauth/complete${location.search}`));
      let json = await result.json();

      if (json.error) {
        throw new Error(json.error);
      }

      auth.setAuth([json.key, new Date(json.expires)]);
    } catch (err) {
      setResult(
        <Redirect to={{
          pathname: "/login",
          state: { error: err.message }
        }} />
      );
    }
  });

  return result ?? <LoadingPage />;
}

export const useAuth = (): AuthContext | null => {
  return useContext(authContext);
};

function useProvideAuth(): AuthContext {
  const [auth, setAuth] = useState(() => {
    let authStorage = getAuthStorage();
    return [authStorage.authKey, authStorage.expires];
  });

  useEffect(() => {
    localStorage.setItem(
      "charteredAuthentication",
      JSON.stringify({ authKey: auth?.[0], expires: auth?.[1] })
    );
  }, [auth]);

  const login = async (username: string, password: string) => {
    let res = await fetch(unauthenticatedEndpoint("login/password"), {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        "User-Agent": window.navigator.userAgent,
      },
      body: JSON.stringify({ username, password }),
    });
    let json = await res.json();

    if (json.error) {
      throw new Error(json.error);
    }

    setAuth([json.key, new Date(json.expires)]);
  };

  const oauthLogin = async (provider: string) => {
    let res = await fetch(unauthenticatedEndpoint(`login/oauth/${provider}/begin`), {
      method: "GET",
      headers: {
        "Content-Type": "application/json",
        "User-Agent": window.navigator.userAgent,
      }
    });
    let json = await res.json();

    if (json.error) {
      throw new Error(json.error);
    }

    window.location.href = json.redirect_url;
  }

  const logout = async () => {
    // todo call the service so we can purge the key from the db
    setAuth(null);
  };

  const getAuthKey = () => {
    if (auth?.[1] > new Date()) {
      return auth[0];
    } else if (auth) {
      return null;
    }
  };

  return {
    login,
    logout,
    getAuthKey,
    oauthLogin,
    setAuth,
  };
}

function getAuthStorage() {
  const saved = localStorage.getItem("charteredAuthentication");
  const initial = JSON.parse(saved);
  return {
    authKey: initial?.authKey || null,
    expires: initial?.expires ? new Date(initial.expires) : null,
  };
}
