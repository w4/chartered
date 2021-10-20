# Configuration Reference

An exhaustive list of all configuration values in chartered.

Configuration files are written in the TOML format, with simple key-value pairs inside of sections (tables). The following
is a quick overview of all settings, with detailed descriptions found below

## chartered-git

### Configuration format

```toml
bind_address = "127.0.0.1:2233"
database_uri = "postgres://user:password@localhost/chartered" # can also be `sqlite://`
```

### Configuration keys

#### `bind_address`
- Type: string

The IP address and port the web server should be bound to.

#### `database_uri`
- Type: string

A connection string for the backing database, either `postgres` or `sqlite`, either in the
format of `postgres://user:password@localhost/chartered` (a [postgres connection URI][pg-uri]),
`sqlite:///path/to/chartered.db` or `sqlite://:memory:`.

[pg-uri]: https://www.postgresql.org/docs/9.4/libpq-connect.html#LIBPQ-CONNSTRING

---

## chartered-web

### Configuration format

```toml
bind_address = "127.0.0.1:8080"
database_uri = "postgres://user:password@localhost/chartered" # can also be `sqlite://`

storage_uri  = "s3://s3-eu-west-1.amazonaws.com/my-cool-crate-store/" # or file:///var/lib/chartered

[auth.password]
enabled = true # enables password auth 

[auth.<provider>] # openid connect provider
enabled = true
discovery_uri = "https://gitlab.com/"
client_id = "[client-id]"
client_secret = "[client-secret]"
```

### Configuration keys

#### `bind_address`
- Type: string

The IP address and port the web server should be bound to.

#### `database_uri`
- Type: string

A connection string for the backing database, either `postgres` or `sqlite`, either in the
format of `postgres://user:password@localhost/chartered` (a [postgres connection URI][pg-uri]),
`sqlite:///path/to/chartered.db` or `sqlite://:memory:`.

[pg-uri]: https://www.postgresql.org/docs/9.4/libpq-connect.html#LIBPQ-CONNSTRING

#### `storage_uri`
- Type: string

A URI in which crates should be stored, this can either be an `s3://` connection URI, or a local file path using
`file://`.

#### `[auth.password]`
The `[auth.password]` table controls the username/password-based authentication method.

##### `enable`
- Type: bool
- Default: false

Enables username/password-based authentication and registration.

#### `[auth.<provider>]`
`[auth.<provider>]` tables represent an OpenID Connect provider that can be used to
login and register to the chartered instance. `<provider>` should not be changed once
set as the value is stored in the database along with users.

##### `enabled`
- Type: bool

Enables the authentication provider, if this is disabled users will not be able to login
nor register using the provider.

##### `discovery_uri`
- Type: bool

The OIDC Discovery URI that can be used to grab configuration for the provider, not including
`/.well-known/openid-configuration`.

##### `client_id`
- Type: string

The Client ID given by the provider to identify the service.

##### `client_secret`
- Type: string

The client secret given by the provider to authenticate the service.
