import Nav from "../sections/Nav";
import { useAuth } from "../useAuth";

export default function NotFound() {
  return (
    <>
      <MaybeShowNav />

      <div
        className="d-flex justify-content-center align-items-center"
        style={{ minHeight: "calc(100vh - 6rem)" }}
      >
        <div className="card border-0 shadow-sm text-black p-2 m-3">
          <div className="card-body">
            Seems you've taken a wrong turn! The page you requested couldn't be
            found.
          </div>
        </div>
      </div>
    </>
  );
}

function MaybeShowNav() {
  const auth = useAuth();

  if (!auth?.getAuthKey()) {
    return <></>;
  }

  return <Nav />;
}
