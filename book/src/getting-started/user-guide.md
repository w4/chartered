# User Guide

Using chartered as a user is actually pretty simple, it's very much like your
standard Cargo registry except with two extra concepts: organisations and
permissions.

Organisations are very much like the organisations you'll likely have used on
your preferred SCM host, a group of users that have group-level permissions,
in Chartered case the permissions these users may have are:

### Permissions

- `VISIBLE`
    - Essentially the base-level permission meaning the user belongs to the group,
      if the user doesn't have this permission they're not in the group, this
      permission at the crate-level means the user can download the crate and see
      it in the WebUI.
- `PUBLISH_VERSION`
    - Gives the ability to publish a new version for crates belonging to the group.
- `YANK_VERSION`
    - Gives the ability to yank (and unyank) published versions for crates belonging
      to the group.
- `MANAGE_USERS`
    - Gives the ability to add (and remove) users from the group, and crates belonging
      to the organisation.
- `CREATE_CRATE`
    - Gives the ability to create a new crate under the organisation.

All these permissions, with the exception of `CREATE_CRATE`, can also be used at the
crate-level for giving extra permissions to org members for a particular crate - or
even users outside of the org. Bare in mind, however, these permissions are _additive_ -
it is not possible for permissions to be _subtracted_ from a user at the crate-level
if they have been granted them by the organisation.

### Publishing your first crate

With all this in mind, it's about time you started publishing your first crate!

Chartered has excellent integration with Cargo's [alternative registry][arp]
implementation and is used very much like a standard alternative registry. The only
prerequisites for publishing your crate are:

1. Have an SSH key added to your Chartered account, which you can do via the WebUI
2. Belong to an organisation you have the `PUBLISH_VERSION` permission for, anyone can
   create a new organisation if you don't already belong to one.

And once you're ready, you can add the organisation's registry to your `.cargo/config.toml`
like so:

```toml
[registries]
my-organisation = { index = "ssh://ssh.your.instance.of.chart.rs/my-organisation" }
```

(You should create this file if it doesn't already exist)

You can now publish the crate using cargo as you normally would, except with the
registry specified:

```sh
$ cargo publish --registry my-organisation --token ""
```

Note: the token is purposefully empty, as the token will be vended by the index based
on your SSH key.

[arp]: https://doc.rust-lang.org/cargo/reference/registries.html

### Pulling in dependencies

Again, not too dissimilar from using [crates.io][cio], you can declare your dependencies
as normal with the exception that you need to specify the organisation the crate should
be pulled from provided you've declared the organisation in `.cargo/config.toml` as shown
in the previous section of this guide.

```toml
[dependencies]
my-other-crate = { version = "0.1", registry = "my-organisation" }
```

Your other Cargo dependencies from [crates.io][cio] can be declared as normal alongside
organisation dependencies.

[cio]: https://crates.io/
