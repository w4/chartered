import React = require("react");
import { useState, useEffect, useContext, createContext } from "react";
import { unauthenticatedEndpoint } from "./util";

export interface AuthContext {
  authKey?: string;
  expires?: Date;
  login: (username: string, password: string) => Promise<void>;
  logout: () => Promise<void>;
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
  const [authKey, setAuthKey] = useState(() => getAuthStorage().authKey);
  const [expires, setExpires] = useState(() => getAuthStorage().expires);

  useEffect(() => {
    localStorage.setItem(
      "charteredAuthentication",
      JSON.stringify({ authKey, expires })
    );
  }, [authKey, expires]);

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

    setExpires(new Date(json.expires));
    setAuthKey(json.key);
  };

  const logout = async () => {
    // todo call the service so we can purge the key from the db
    localStorage.removeItem("charteredAuthentication");
    setExpires(null);
    setAuthKey(null);
  };

  return {
    authKey,
    expires,
    login,
    logout,
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
