# chartered

a little dig at creating a private cargo repository with authenticated downloads, the plan is to have git connect to
a git server we setup that we can serve a fake index from generated just for the authenticated user that we can embed
authentication credentials into.

[open tasks](https://github.com/w4/chartered/issues)

#### fine grained permissions per user per crate

- VISIBLE
- PUBLISH_VERSION
- YANK_VERSION
- MANAGE_USERS

(support for groups coming)
