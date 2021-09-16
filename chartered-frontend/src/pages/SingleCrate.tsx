import React = require('react');

import { useState, useEffect } from 'react';

import { Link } from "react-router-dom";
import { useAuth } from '../useAuth';
import Nav from "../sections/Nav";
import { Box, HouseDoor, Book, Building, PersonPlus } from 'react-bootstrap-icons';
import { useParams } from "react-router-dom";
import { authenticatedEndpoint } from '../util';

import Prism from 'react-syntax-highlighter/dist/cjs/prism';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';

type Tab = 'readme' | 'versions' | 'members';

export default function SingleCrate() {
    const auth = useAuth();
    const { crate } = useParams();

    const [crateInfo, setCrateInfo] = useState(null);
    const [currentTab, setCurrentTab] = useState<Tab>('readme');

    useEffect(async () => {
        let res = await fetch(authenticatedEndpoint(auth, `crates/${crate}`));
        let json = await res.json();
        setCrateInfo(json);
    }, []);

    if (!crateInfo) {
        return (<div>Loading...</div>);
    }

    const crateVersion = crateInfo.versions[crateInfo.versions.length - 1];

    return (
        <div className="text-white">
            <Nav />

            <div className="container mt-4 pb-4">
                <div className="row align-items-stretch">
                    <div className="col-md-6">
                        <div className="card border-0 shadow-sm text-black h-100">
                            <div className="card-body">
                                <div className="d-flex flex-row align-items-center">
                                    <div className="text-white circle bg-primary bg-gradient d-inline rounded-circle d-inline-flex justify-content-center align-items-center"
                                        style={{ width: '2rem', height: '2rem' }}>
                                        <Box />
                                    </div>
                                    <h1 className="text-primary d-inline px-2">{crate}</h1>
                                    <h2 className="text-secondary m-0">{crateVersion.vers}</h2>
                                </div>

                                <p className="m-0">{crateVersion.description}</p>
                            </div>
                        </div>
                    </div>

                    <div className="col-md-6">
                        <div className="card border-0 shadow-sm text-black h-100">
                            <div className="card-body">
                                <HouseDoor /> <a href={crateVersion.homepage}>{crateVersion.homepage}</a><br />
                                <Book /> <a href={crateVersion.documentation}>{crateVersion.documentation}</a><br />
                                <Building /> <a href={crateVersion.repository}>{crateVersion.repository}</a>
                            </div>
                        </div>
                    </div>
                </div>

                <div className="row my-4">
                    <div className="col-md-9">
                        <div className="card border-0 shadow-sm text-black">
                            <div className="card-header">
                                <ul className="nav nav-pills card-header-pills">
                                    <li className="nav-item">
                                        <a className={`nav-link ${currentTab == 'readme' ? 'bg-primary bg-gradient active' : ''}`} href="#"
                                            onClick={() => setCurrentTab('readme')}>
                                            Readme
                                        </a>
                                    </li>
                                    <li className="nav-item">
                                        <a className={`nav-link ${currentTab == 'versions' ? 'bg-primary bg-gradient active' : ''}`} href="#"
                                            onClick={() => setCurrentTab('versions')}>
                                            Versions
                                            <span className={`badge rounded-pill bg-danger ms-1`}>{crateInfo.versions.length}</span>
                                        </a>
                                    </li>
                                    <li className="nav-item">
                                        <a className={`nav-link ${currentTab == 'members' ? 'bg-primary bg-gradient active' : ''}`} href="#"
                                            onClick={() => setCurrentTab('members')}>
                                            Members
                                        </a>
                                    </li>
                                </ul>
                            </div>

                            <div className="card-body">
                                {currentTab == 'readme' ? <ReadMe crateInfo={crateVersion} /> : <></>}
                                {currentTab == 'versions' ? <>Versions</> : <></>}
                                {currentTab == 'members' ? <Members crateInfo={crateVersion} /> : <></>}
                            </div>
                        </div>
                    </div>

                    <div className="col-md-3">
                        <div className="card border-0 shadow-sm text-black">
                            <div className="card-body pb-0">
                                <h5 className="card-title">Dependencies</h5>
                            </div>

                            <ul className="list-group list-group-flush mb-2">
                                {crateVersion.deps.map(dep => (
                                    <li key={`${dep.name}-${dep.version_req}`} className="list-group-item">{dep.name} = "<strong>{dep.version_req}</strong>"</li>
                                ))}
                            </ul>
                        </div>

                        <div className="card border-0 shadow-sm text-black mt-4">
                            <div className="card-body pb-0">
                                <h5 className="card-title">Dependents</h5>
                            </div>

                            <ul className="list-group list-group-flush">
                                <li className="list-group-item">An item</li>
                                <li className="list-group-item">A second item</li>
                                <li className="list-group-item">A third item</li>
                            </ul>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    );
}

function ReadMe(props: { crateInfo: any }) {
    return (
        <ReactMarkdown children={props.crateInfo.readme} remarkPlugins={[remarkGfm]} components={{
            code({node, inline, className, children, ...props}) {
                const match = /language-(\w+)/.exec(className || '')
                return !inline && match ? (
                <Prism
                    children={String(children).replace(/\n$/, '')}
                    language={match[1]}
                    PreTag="pre"
                    {...props}
                />
                ) : (
                <code className={className} {...props}>
                    {children}
                </code>
                )
            }
        }} />
    );
}

function Members(props: { crateInfo: any }) {
    return <div className="grid" style={{ gridTemplateColumns: '1fr 1fr 1fr' }}>
        <div className="g-col-4 g-col-lg-1 d-flex align-items-center">
            <img src="http://placekitten.com/96/96" className="rounded-circle" />

            <div className="ms-2">
                <strong>Johnny Davidson</strong> <em>(that's you!)</em><br />
                Owner
            </div>
        </div>

        <div className="g-col-4 g-col-lg-1 d-flex align-items-center mt-2">
            <img src="http://placekitten.com/96/96" className="rounded-circle" />

            <div className="ms-2">
                <strong>Will Woodwood</strong><br />
                <select className="form-select form-select-sm" aria-label="Default select example">
                    <option value="1">Consumer</option>
                    <option value="2">Maintainer</option>
                    <option value="3">Owner</option>
                </select>
            </div>
        </div>

        <div className="g-col-4 g-col-lg-1 d-flex align-items-center mt-2">
            <img src="http://placekitten.com/96/96" className="rounded-circle" />

            <div className="ms-2">
                <strong>Ben Dover</strong><br />
                <select className="form-select form-select-sm" aria-label="Default select example">
                    <option value="1">Consumer</option>
                    <option value="2" selected>Maintainer</option>
                    <option value="3">Owner</option>
                </select>
            </div>
        </div>

        <div className="g-col-4 g-col-lg-1 d-flex align-items-center mt-2">
            <img src="http://placekitten.com/96/96" className="rounded-circle" />

            <div className="ms-2">
                <strong>Eline Dover</strong><br />
                <select className="form-select form-select-sm" aria-label="Default select example">
                    <option value="1">Consumer</option>
                    <option value="2">Maintainer</option>
                    <option value="3" selected>Owner</option>
                </select>
            </div>
        </div>

        <div className="g-col-4 g-col-lg-1 mt-2 d-flex align-items-center justify-content-center rounded-circle"
            style={{ width: '96px', height: '96px', background: '#DEDEDE', fontSize: '2rem' }}>
            <PersonPlus />
        </div>
    </div>;
}