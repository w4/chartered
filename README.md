# chartered

a little dig at creating a private cargo repository with authenticated downloads, the plan is to have git connect to
a git server we setup that we can serve a fake index from generated just for the authenticated user that we can embed
authentication credentials into.

i've got git connecting to this server and attempting to communicate with it after sending a little bit of hard-coded
preamble.

next steps:

- ~reverse engineer & create tokio codec for the git protocol~ **now successfully generating a repo, tree & blob in memory git clients can understand!**
- clean up all the hacked-together code and package it all up into a nice library for generating git commit objects etc
- serve an index
- serve cargo manifest over git (how does git handle 'force pushes' from server -> client? lets see how they like it for once, i'm sick of people picking on servers all the time)
- serve .crate files over http using auth tokens we generated while serving the manifest

#### open q's

maybe it'd be better to use git directly and create an index on the filesystem just for the user? seems less fun though
