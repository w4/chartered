CREATE TABLE users (
    id INTEGER PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    uuid BYTEA NOT NULL UNIQUE,
    username VARCHAR(255) NOT NULL UNIQUE,
    password CHAR(60),
    name VARCHAR(255),
    nick VARCHAR(255),
    email VARCHAR(255),
    external_profile_url VARCHAR(2048),
    picture_url VARCHAR(2048)
);

CREATE TABLE organisations (
    id INTEGER PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    uuid BYTEA NOT NULL UNIQUE,
    name VARCHAR(255) NOT NULL UNIQUE,
    description TEXT NOT NULL DEFAULT '',
    public BOOLEAN NOT NULL DEFAULT FALSE
);

CREATE TABLE crates (
    id INTEGER PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    name VARCHAR(255) NOT NULL,
    organisation_id INTEGER NOT NULL,
    readme TEXT,
    description VARCHAR(255),
    repository VARCHAR(255),
    homepage VARCHAR(255),
    documentation VARCHAR(255),
    downloads INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (name, organisation_id),
    FOREIGN KEY (organisation_id) REFERENCES organisations (id)
);

CREATE TABLE crate_versions (
    id INTEGER PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    crate_id INTEGER NOT NULL,
    version VARCHAR(255) NOT NULL,
    filesystem_object VARCHAR(255) NOT NULL,
    size INTEGER NOT NULL,
    yanked BOOLEAN NOT NULL DEFAULT FALSE,
    checksum VARCHAR(255) NOT NULL,
    dependencies BYTEA NOT NULL,
    features BYTEA NOT NULL,
    links VARCHAR(255),
    user_id INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (crate_id, version),
    FOREIGN KEY (user_id) REFERENCES users (id),
    FOREIGN KEY (crate_id) REFERENCES crates (id)
);

CREATE TABLE user_organisation_permissions (
    id INTEGER PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    user_id INTEGER NOT NULL,
    organisation_id INTEGER NOT NULL,
    permissions INTEGER NOT NULL,
    UNIQUE (user_id, organisation_id),
    FOREIGN KEY (user_id) REFERENCES users (id),
    FOREIGN KEY (organisation_id) REFERENCES organisations (id)
);

CREATE TABLE user_crate_permissions (
    id INTEGER PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    user_id INTEGER NOT NULL,
    crate_id INTEGER NOT NULL,
    permissions INTEGER NOT NULL,
    UNIQUE (user_id, crate_id),
    FOREIGN KEY (user_id) REFERENCES users (id),
    FOREIGN KEY (crate_id) REFERENCES crates (id)
);

CREATE TABLE user_ssh_keys (
    id INTEGER PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    uuid BYTEA NOT NULL UNIQUE,
    name VARCHAR(255) NOT NULL,
    user_id INTEGER NOT NULL,
    ssh_key BYTEA NOT NULL UNIQUE,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_used_at TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users (id)
);

CREATE TABLE user_sessions (
    id INTEGER PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    user_id INTEGER NOT NULL,
    session_key VARCHAR(255) NOT NULL UNIQUE,
    user_ssh_key_id INTEGER,
    expires_at TIMESTAMP,
    user_agent VARCHAR(255),
    ip VARCHAR(255),
    FOREIGN KEY (user_id) REFERENCES users (id),
    FOREIGN KEY (user_ssh_key_id) REFERENCES user_ssh_keys (id)
);

CREATE TABLE server_private_keys (
     id INTEGER PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
     ssh_key_type VARCHAR(255) NOT NULL UNIQUE,
     ssh_private_key BYTEA NOT NULL UNIQUE
);
