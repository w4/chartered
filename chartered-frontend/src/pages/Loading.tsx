import React = require("react");

export function LoadingSpinner() {
  return (
    <div className="p-4 d-flex justify-content-center align-items-center">
      <div className="spinner-border text-primary" role="status">
        <span className="visually-hidden">Loading...</span>
      </div>
    </div>
  );
}

export default function LoadingPage() {
  return (
    <div className="min-vh-100 bg-primary d-flex justify-content-center align-items-center">
      <div className="spinner-border text-light" role="status">
        <span className="visually-hidden">Loading...</span>
      </div>
    </div>
  );
}
