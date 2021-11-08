import { useState, useEffect, useContext, createContext } from "react";
import { useLocation, Navigate, useNavigate } from "react-router-dom";
import {
  authenticatedEndpoint,
  BASE_URL,
  unauthenticatedEndpoint,
} from "./util";
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

interface ExtendResponse {
  expires: number;
  error?: string;
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
  const navigate = useNavigate();

  useEffect(async () => {
    try {
      let result = await fetch(
        unauthenticatedEndpoint(`auth/login/oauth/complete${location.search}`)
      );
      let json = await result.json();

      auth?.handleLoginResponse(json);
    } catch (err: any) {
      navigate("/login", {
        state: { prompt: { message: err.message, kind: "danger" } },
      });
    }
  });

  return <LoadingPage />;
}

export const useAuth = (): AuthContext | null => {
  return useContext(authContext);
};

function useProvideAuth(): AuthContext {
  const [auth, setAuth] = useState<any[] | null>(() => {
    let authStorage = getAuthStorage();
    return [
      authStorage.userUuid,
      authStorage.authKey,
      authStorage.expires,
      authStorage.pictureUrl,
    ];
  });

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
    let res = await fetch(unauthenticatedEndpoint("auth/login/password"), {
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
      unauthenticatedEndpoint(`auth/login/oauth/${provider}/begin`),
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

  const getAuthKey = () => {
    if (auth?.[2] > new Date()) {
      return auth?.[1];
    } else if (auth) {
      return null;
    }
  };

  const logout = async () => {
    if (auth === null) {
      return;
    }

    try {
      await fetch(`${BASE_URL}/a/${getAuthKey()}/web/v1/auth/logout`, {
        method: "GET",
        headers: {
          "Content-Type": "application/json",
          "User-Agent": window.navigator.userAgent,
        },
      });
    } catch (e) {
      console.error("Failed to fully log user out of session", e);
    } finally {
      setAuth(null);
    }
  };

  const extendSession = async () => {
    if (auth === null || auth?.[2] < new Date()) {
      return;
    }

    try {
      const res = await fetch(
        `${BASE_URL}/a/${getAuthKey()}/web/v1/auth/extend`,
        {
          method: "GET",
          headers: {
            "Content-Type": "application/json",
            "User-Agent": window.navigator.userAgent,
          },
        }
      );
      const response: ExtendResponse = await res.json();

      if (response.error) {
        throw new Error(response.error);
      }

      const newAuth = [...auth];
      newAuth[2] = new Date(response.expires);
      setAuth(newAuth);
    } catch (e) {
      console.error("Failed to extend user session", e);
    }
  };

  const getUserUuid = () => {
    if (auth?.[2] > new Date()) {
      return auth?.[0];
    } else if (auth) {
      return null;
    }
  };

  const getPictureUrl = () => {
    if (auth?.[2] > new Date()) {
      return auth?.[3];
    } else if (auth) {
      return null;
    }
  };

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

    const extendInterval = 60000;
    if (auth?.[2] && auth?.[2].getTime() - new Date().getTime() <= 0) {
      extendSession();
    }

    const extendSessionIntervalId = setInterval(
      () => extendSession(),
      extendInterval
    );

    return () => clearInterval(extendSessionIntervalId);
  }, [auth]);

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
  const initial = saved ? JSON.parse(saved) : {};
  return {
    userUuid: initial?.userUuid || null,
    authKey: initial?.authKey || null,
    expires: initial?.expires ? new Date(initial.expires) : null,
    pictureUrl: initial?.pictureUrl,
  };
}
