CREATE TABLE crates (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    name VARCHAR(255) NOT NULL UNIQUE
);

CREATE TABLE crate_versions (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    crate_id INTEGER NOT NULL,
    version VARCHAR(255) NOT NULL,
    filesystem_object VARCHAR(255) NOT NULL,
    yanked BOOLEAN NOT NULL DEFAULT FALSE,
    checksum VARCHAR(255) NOT NULL,
    dependencies BLOB NOT NULL,
    features BLOB NOT NULL,
    links VARCHAR(255),
    UNIQUE (crate_id, version),
    FOREIGN KEY (crate_id) REFERENCES crates (id)
);

CREATE TABLE users (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    username VARCHAR(255) NOT NULL UNIQUE
);

CREATE TABLE user_ssh_keys (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    ssh_key BLOB NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users (id)
);

CREATE TABLE user_api_keys (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    api_key VARCHAR(255) NOT NULL UNIQUE,
    user_ssh_key_id INTEGER,
    expires_at DATETIME,
    FOREIGN KEY (user_id) REFERENCES users (id)
    FOREIGN KEY (user_ssh_key_id) REFERENCES user_ssh_keys (id)
);

CREATE TABLE user_crate_permissions (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    crate_id INTEGER NOT NULL,
    permissions INTEGER NOT NULL,
    UNIQUE (user_id, crate_id),
    FOREIGN KEY (user_id) REFERENCES users (id)
    FOREIGN KEY (crate_id) REFERENCES crates (id)
);
