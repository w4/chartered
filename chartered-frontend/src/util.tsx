import React = require("react");
import ReactPlaceholder from "react-placeholder";
import { AuthContext } from "./useAuth";

export const BASE_URL = process.env.BASE_URL || "http://localhost:8888";

export function unauthenticatedEndpoint(endpoint: string): string {
  return `${BASE_URL}/a/-/web/v1/${endpoint}`;
}

export function authenticatedEndpoint(
  auth: AuthContext,
  endpoint: string
): string {
  return `${BASE_URL}/a/${auth.getAuthKey()}/web/v1/${endpoint}`;
}

export function useAuthenticatedRequest<S>(
  { auth, endpoint }: { auth: AuthContext; endpoint: string },
  reloadOn = []
): { response: S | null; error: string | null } {
  const [error, setError] = React.useState(null);
  const [response, setResponse] = React.useState(null);

  React.useEffect(async () => {
    try {
      setResponse(null);
      let res = await fetch(authenticatedEndpoint(auth, endpoint));

      if (res.status == 401) {
        await auth.logout();
        return null;
      }

      let jsonRes = await res.json();

      if (jsonRes.error) {
        setError(jsonRes.error);
      } else {
        setResponse(jsonRes);
      }
    } catch (e) {
      setError(e.message);
    }
  }, reloadOn);

  return { response, error };
}

export function useUnauthenticatedRequest<S>(
  { endpoint }: { endpoint: string },
  reloadOn = []
): { response: S | null; error: string | null } {
  const [error, setError] = React.useState(null);
  const [response, setResponse] = React.useState(null);

  React.useEffect(async () => {
    try {
      let res = await fetch(unauthenticatedEndpoint(endpoint));
      let jsonRes = await res.json();

      if (jsonRes.error) {
        setError(jsonRes.error);
      } else {
        setResponse(jsonRes);
      }
    } catch (e) {
      setError(e.message);
    }
  }, reloadOn);

  return { response, error };
}

export function ProfilePicture({
  src,
  height,
  width,
  className,
}: {
  src: string;
  height: string;
  width: string;
  className?: string;
}) {
  if (src !== null) {
    return (
      <RoundedPicture
        src={src}
        height={height}
        width={width}
        className={className}
      />
    );
  } else {
    return (
      <div
        className={`rounded-circle d-inline-block ${className}`}
        style={{ width, height, background: "rgb(235, 235, 235)" }}
      />
    );
  }
}

export function RoundedPicture({
  src,
  height,
  width,
  className,
}: {
  src: string;
  height: string;
  width: string;
  className?: string;
}) {
  const [imageLoaded, setImageLoaded] = React.useState(false);

  return (
    <div
      className={`position-relative d-inline-block ${className || ""}`}
      style={{ height, width }}
    >
      <ReactPlaceholder
        showLoadingAnimation
        type="round"
        style={{ height, width, position: "absolute" }}
        ready={imageLoaded}
      >
        <></>
      </ReactPlaceholder>
      <img
        style={{
          visibility: imageLoaded ? "visible" : "hidden",
          height,
          width,
        }}
        src={src}
        onLoad={() => setImageLoaded(true)}
        className="rounded-circle"
      />
    </div>
  );
}
