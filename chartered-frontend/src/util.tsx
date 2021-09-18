import React = require("react");
import { AuthContext } from "./useAuth";

export const BASE_URL = "http://localhost:8888";

export function unauthenticatedEndpoint(endpoint: string): string {
  return `${BASE_URL}/a/-/web/v1/${endpoint}`;
}

export function authenticatedEndpoint(
  auth: AuthContext,
  endpoint: string
): string {
  return `${BASE_URL}/a/${auth.authKey}/web/v1/${endpoint}`;
}

export function useAuthenticatedRequest<S>(
  { auth, endpoint }: { auth: AuthContext; endpoint: string },
  reloadOn = []
): { response: S | null; error: string | null } {
  const [error, setError] = React.useState(null);
  const [response, setResponse] = React.useState(null);

  React.useEffect(async () => {
    try {
      let req = await fetch(authenticatedEndpoint(auth, endpoint));
      let res = await req.json();

      if (res.error) {
        setError(res.error);
      } else {
        setResponse(res);
      }
    } catch (e) {
      setError(e.message);
    }
  }, reloadOn);

  return { response, error };
}
