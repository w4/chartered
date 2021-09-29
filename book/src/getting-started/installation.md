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
storage for crate lookups and authentication credential vending.

### Backend Services

`chartered-git` and `chartered-web`'s setups are similar, first they need a database set up -
either SQLite or PostgreSQL, PostgreSQL is recommended anywhere outside of development/local
use for obvious reasons.

Both the aformentioned services have sane defaults for development and can be ran simply by
running the binary, the services will bind to `127.0.0.1:8899` and `127.0.0.1:8080` respectively
and store crate files in `/tmp/chartered`, configuration away from these defaults is simple.

Using the recommended setup, S3 & PostgreSQL:

```toml
bind-address = "127.0.0.1:8080"
database-uri = "postgres://user:password@localhost/chartered"

# unlike the above two options, this configuration value should only be supplied
# for chartered-web
crate-store  = "s3://s3-eu-west-1.amazonaws.com/my-cool-crate-store/"
```

Or, using the defaults as an example:

```toml
bind-address = "127.0.0.1:8899"
database-uri = "sqlite://chartered.db"
crate-store  = "file:///tmp/chartered"
```

These configuration files can be passed into each binary using the `-c` CLI argument.

Alternative crate stores will be considered, please consider contributing or
[create an issue on GitHub][gh-issue]. <span style="color: transparent;">MySQL support, however, is a no-go.</span>

[gh-issue]: https://github.com/w4/chartered/issues

### Frontend

The frontend only needs to be configured to point to the `chartered-web` service. This can be
done by changing the bundled `config.json`. This can then be hosted in S3/minio/your preferred
static hosting platform, a Dockerfile can also be built which uses [`static-web-server`][sws]
to run on your own server without another way of hosting static content:

```sh
$ DOCKER_BUILDKIT=0 docker build https://github.com/w4/chartered.git#main:chartered-frontend \
    --build-arg BASE_URL=https://my.instance.chart.rs -t chartered-frontend:master
$ docker run -p 8080:80 chartered-frontend:master
$ curl http://127.0.0.1:8080
<!DOCTYPE html><html lang="en"><head><meta charset="UTF-8">...
```

[sws]: https://github.com/joseluisq/static-web-server
