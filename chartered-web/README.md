# chartered-web (binary)

The web UI and Cargo HTTP API implementation for chartered. The Cargo
HTTP API is authenticated using a key either generated during login on
the Web UI or embedded into the cargo index returned by `chartered-git`.

The web UI also allows for adding SSH keys that can be used to
authenticate and identify yourself to `chartered-git`, it's also used
for group management, etc.
