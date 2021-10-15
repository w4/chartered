import React = require("react");
import { useState, useEffect, useContext, createContext } from "react";
import { useLocation, Redirect } from "react-router-dom";
import { unauthenticatedEndpoint } from "./util";
import LoadingPage from "./pages/Loading";

export interface OAuthProviders {
  providers: string[];
}

interface LoginResponse {
  user_uuid: string;
  key: string;
  expires: number;
  error?: string;
  picture_url?: string;
}

export interface AuthContext {
  login: (username: string, password: string) => Promise<void>;
  oauthLogin: (provider: string) => Promise<void>;
  logout: () => Promise<void>;
  getAuthKey: () => Promise<string | null>;
  getUserUuid: () => string;
  getPictureUrl: () => string;
  handleLoginResponse: (json: LoginResponse) => any;
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
      let result = await fetch(
        unauthenticatedEndpoint(`login/oauth/complete${location.search}`)
      );
      let json = await result.json();

      auth.handleLoginResponse(json);
    } catch (err) {
      setResult(
        <Redirect
          to={{
            pathname: "/login",
            state: { error: err.message },
          }}
        />
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
    return [
      authStorage.userUuid,
      authStorage.authKey,
      authStorage.expires,
      authStorage.pictureUrl,
    ];
  });

  useEffect(() => {
    localStorage.setItem(
      "charteredAuthentication",
      JSON.stringify({
        userUuid: auth?.[0],
        authKey: auth?.[1],
        expires: auth?.[2],
        pictureUrl: auth?.[3],
      })
    );
  }, [auth]);

  const handleLoginResponse = (response: LoginResponse) => {
    if (response.error) {
      throw new Error(response.error);
    }

    setAuth([
      response.user_uuid,
      response.key,
      new Date(response.expires),
      response.picture_url,
    ]);
  };

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

    handleLoginResponse(json);
  };

  const oauthLogin = async (provider: string) => {
    let res = await fetch(
      unauthenticatedEndpoint(`login/oauth/${provider}/begin`),
      {
        method: "GET",
        headers: {
          "Content-Type": "application/json",
          "User-Agent": window.navigator.userAgent,
        },
      }
    );
    let json = await res.json();

    if (json.error) {
      throw new Error(json.error);
    }

    window.location.href = json.redirect_url;
  };

  const logout = async () => {
    // todo call the service so we can purge the key from the db
    setAuth(null);
  };

  const getAuthKey = () => {
    if (auth?.[2] > new Date()) {
      return auth[1];
    } else if (auth) {
      return null;
    }
  };

  const getUserUuid = () => {
    if (auth?.[2] > new Date()) {
      return auth[0];
    } else if (auth) {
      return null;
    }
  };

  const getPictureUrl = () => {
    if (auth?.[2] > new Date()) {
      return auth[3];
    } else if (auth) {
      return null;
    }
  };

  return {
    login,
    logout,
    getAuthKey,
    getUserUuid,
    getPictureUrl,
    oauthLogin,
    handleLoginResponse,
  };
}

function getAuthStorage() {
  const saved = localStorage.getItem("charteredAuthentication");
  const initial = JSON.parse(saved);
  return {
    userUuid: initial?.userUuid || null,
    authKey: initial?.authKey || null,
    expires: initial?.expires ? new Date(initial.expires) : null,
    pictureUrl: initial?.pictureUrl,
  };
}
