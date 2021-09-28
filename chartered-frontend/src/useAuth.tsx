import React = require("react");
import { useState, useEffect, useContext, createContext } from "react";
import { unauthenticatedEndpoint } from "./util";

export interface AuthContext {
  login: (username: string, password: string) => Promise<void>;
  logout: () => Promise<void>;
  getAuthKey: () => Promise<string | null>;
}

const authContext = createContext<AuthContext | null>(null);

export function ProvideAuth({ children }: { children: any }) {
  const auth = useProvideAuth();
  return <authContext.Provider value={auth}>{children}</authContext.Provider>;
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
    let res = await fetch(unauthenticatedEndpoint("login"), {
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
