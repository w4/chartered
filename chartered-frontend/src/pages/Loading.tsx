import React = require("react");

export default function Loading() {
    return <div className="min-vh-100 bg-primary d-flex justify-content-center align-items-center">
        <div className="spinner-border text-light" role="status">
            <span className="visually-hidden">Loading...</span>
        </div>
    </div>;
}