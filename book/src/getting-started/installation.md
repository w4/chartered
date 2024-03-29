# Server Installation

Chartered's server comes in 3 parts:

- **chartered-git**: hosts the git server which clients grab the crate index from, along with
  their credentials to grab the crates from the next service,
- **chartered-web**: hosts the API portion of chartered, which serves the crates themselves
  (or a redirect to them, depending on which storage backend you're using) and hosts the "web
  API" which is consumed by our final service,
- **chartered-frontend**: a React-based [crates.io](https://crates.io/)-like web UI for viewing
  crates, managing organisations and viewing AAA data.

Each of these services are hosted separately from one another, and could technically be swapped
out for other implementations - the only shared layer between the three of them is database
storage for crate lookups and authentication credential vending. All of the services have the
ability to be clustered with no extra configuration.

### Backend Services

`chartered-git` and `chartered-web`'s setups are similar, first they need a database set up -
either SQLite or PostgreSQL, PostgreSQL is recommended anywhere outside of development/local
use for obvious reasons.

Both the aformentioned services have sane defaults for development and can be ran simply by
running the binary, the services will bind to `127.0.0.1:8899` and `127.0.0.1:8080` respectively
and store crate files in `/tmp/chartered`, configuration away from these defaults is simple.

Using the recommended setup, S3 & PostgreSQL:


#### `chartered-web` config

```toml
bind_address = "127.0.0.1:8080"
database_uri = "postgres://user:password@localhost/chartered"
storage_uri  = "s3://s3-eu-west-1.amazonaws.com/my-cool-crate-store/"
frontend_base_uri = "http://localhost:5173/"

[auth.password]
enabled = true

# openid connect provider
[auth.gitlab]
enabled = true
discovery_uri = "https://gitlab.com/"
client_id = "[client-id]"
client_secret = "[client-secret]"
```

#### `chartered-git` config

```toml
bind_address = "127.0.0.1:2233"
database_uri = "postgres://user:password@localhost/chartered" # can also be `sqlite://`
web_base_uri = "http://localhost:8888/"

[committer]
name = "Chartered"
email = "noreply@chart.rs"
message = "Updated crates!"
```

These configuration files can be passed into each binary using the `-c` CLI argument.

Alternative crate stores will be considered, please consider contributing or
[create an issue on GitHub][gh-issue]. <span style="color: transparent;">MySQL support, however, is a no-go.</span>

`chartered-web` & `chartered-git` can be built from source easily or ran using the
Dockerfile:

```
$ docker build https://github.com/w4/chartered.git#main \
    --target chartered-web \
    -t chartered-web:master
$ docker build https://github.com/w4/chartered.git#main \
    --target chartered-git \
    -t chartered-git:master
$ docker -v $PWD/web-config.toml:/config.toml run -d chartered-web --config /config.toml
$ docker -v $PWD/git-config.toml:/config.toml run -d chartered-git --config /config.toml
```

[gh-issue]: https://github.com/w4/chartered/issues

### Frontend

The frontend only needs to be configured to point to the `chartered-web` service. This can be
done by changing the bundled `config.json`. This can then be hosted in S3/minio/your preferred
static hosting platform, a Dockerfile can also be built which uses [`static-web-server`][sws]
to run on your own server without another way of hosting static content:

```sh
# buildkit doesn't yet support subdirectories for git repositories
$ DOCKER_BUILDKIT=0 docker build \
    https://github.com/w4/chartered.git#main:chartered-frontend \
    --build-arg BASE_URL=https://api.my.instance.chart.rs \
    -t chartered-frontend:master
$ docker run -d -p 8080:80 chartered-frontend:master
$ curl http://127.0.0.1:8080
<!DOCTYPE html><html lang="en"><head><meta charset="UTF-8">...
```

Where `BASE_URL` points to the `chartered-web` instance.

[sws]: https://github.com/joseluisq/static-web-server
