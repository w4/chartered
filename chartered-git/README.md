# chartered-git (binary)

Spins up an isolated SSH server, accepting connections only from those
with known SSH keys and serves the git protocol over it. When requesting
the git packfile, it'll serve a cargo index with only packages the user
has access to in there.

To ensure proper authorization when the user pulls the `.crate` file
over the HTTP API, the cargo index's `config.json` will have credentials
embedded into it specific to the user, which will be passed by cargo to
the HTTP API.

The user is only served a single, detatched commit from this git server,
cargo handles this gracefully as it checks out the server's HEAD rather
than trying to merge it into a master branch.
