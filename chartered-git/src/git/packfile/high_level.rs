//! A high-level interface for building packfiles. Wraps the `low_level` module
//! making a much easier interface for writing files and generating the root
//! commit.
//!
//! The output packfile will only have a single commit in it, which is fine
//! for our purposes because `cargo` will `git pull --force` from our Git
//! server, allowing us to ignore any history the client may have.

use arrayvec::ArrayVec;
use indexmap::IndexMap;

use super::low_level::{
    Commit, CommitUserInfo, HashOutput, PackFileEntry, TreeItem as LowLevelTreeItem, TreeItemKind,
};

/// The main way of interacting with the high level Packfile builder
///
/// Builds a whole packfile containing files, directories and commits - essentially
/// building out a full Git repository in memory.
#[derive(Debug)]
pub struct GitRepository<'a> {
    /// A map containing all the blobs and their corresponding hashes so they're
    /// not inserted more than once for any files in the whole tree with the same
    /// content.
    packfile_entries: IndexMap<HashOutput, PackFileEntry<'a>>,
    /// An in-progress `Tree` currently being built out, the tree refers to items
    /// in `file_entries` by hash.
    tree: Tree<'a>,
}

impl Default for GitRepository<'_> {
    fn default() -> Self {
        Self {
            packfile_entries: IndexMap::new(),
            tree: Tree::default(),
        }
    }
}

impl<'a> GitRepository<'a> {
    /// Inserts a file into the repository, writing a file to the path
    /// `path/to/my-file` would require a `path` of `["path", "to"]`
    /// and a `file` of `"my-file"`.
    pub fn insert<const N: usize>(
        &mut self,
        path: ArrayVec<&'a str, N>,
        file: &'a str,
        content: &'a [u8],
    ) -> Result<(), anyhow::Error> {
        // we'll initialise the directory to the root of the tree, this means
        // if a path isn't specified we'll just write it to the root directory
        let mut directory = &mut self.tree;

        // loops through the parts in the path, recursing through the `directory`
        // `Tree` until we get to our target directory, creating any missing
        // directories along the way.
        for part in path {
            let tree_item = directory
                .0
                .entry(part)
                .or_insert_with(|| Box::new(TreeItem::Tree(Tree::default())));

            if let TreeItem::Tree(d) = tree_item.as_mut() {
                directory = d;
            } else {
                // TODO: how should we handle this? one of items we tried to
                //  recurse into was a directory.
                anyhow::bail!("attempted to use a file as a directory");
            }
        }

        // wrap the file in a Blob so it's ready for writing into the packfile, and also
        // allows us to grab the hash of the file for use in the tree
        let entry = PackFileEntry::Blob(content);
        let file_hash = entry.hash()?;

        // todo: what should we do on overwrite?
        directory
            .0
            .insert(file, Box::new(TreeItem::Blob(file_hash)));

        self.packfile_entries.insert(file_hash, entry);

        Ok(())
    }

    /// Finalises this `GitRepository` by writing a commit to the `packfile_entries`,
    /// all the files currently in the `tree`, returning all the packfile entries
    /// and also the commit hash so it can be referred to by `ls-ref`s.
    pub fn commit(
        &'a mut self,
        name: &'a str,
        email: &'a str,
        message: &'a str,
    ) -> Result<(HashOutput, Vec<PackFileEntry<'a>>), anyhow::Error> {
        // gets the hash of the entire tree from the root
        let tree_hash = self.tree.to_packfile_entries(&mut self.packfile_entries)?;

        // build the commit using the given inputs
        let commit_user = CommitUserInfo {
            name,
            email,
            time: chrono::Utc::now(),
        };

        let commit = PackFileEntry::Commit(Commit {
            tree: tree_hash,
            author: commit_user,
            committer: commit_user,
            message,
        });

        // write the commit out to the packfile_entries
        let commit_hash = commit.hash()?;
        self.packfile_entries.insert(commit_hash, commit);

        // TODO: make PackFileEntry copy and remove this clone
        Ok((
            commit_hash,
            self.packfile_entries.values().cloned().collect(),
        ))
    }
}

/// An in-progress tree builder, containing file hashes along with their names or nested trees
#[derive(Default, Debug)]
struct Tree<'a>(IndexMap<&'a str, Box<TreeItem<'a>>>);

impl<'a> Tree<'a> {
    /// Recursively writes the the whole tree out to the given `pack_file`,
    /// the tree contains pointers to (hashes of) files contained within a
    /// directory, and pointers to other directories.
    fn to_packfile_entries(
        &self,
        pack_file: &mut IndexMap<HashOutput, PackFileEntry<'a>>,
    ) -> Result<HashOutput, anyhow::Error> {
        let mut tree = Vec::with_capacity(self.0.len());

        for (name, item) in &self.0 {
            tree.push(match item.as_ref() {
                TreeItem::Blob(hash) => LowLevelTreeItem {
                    kind: TreeItemKind::File,
                    name,
                    hash: *hash,
                },
                TreeItem::Tree(tree) => LowLevelTreeItem {
                    kind: TreeItemKind::Directory,
                    name,
                    // we're essentially working through our tree from the bottom up,
                    // so we can grab the hash of each directory along the way and
                    // reference it from the parent directory
                    hash: tree.to_packfile_entries(pack_file)?,
                },
            });
        }

        // gets the hash of the tree we've just worked on, and
        // pushes it to the packfile
        let tree = PackFileEntry::Tree(tree);
        let hash = tree.hash()?;
        pack_file.insert(hash, tree);

        Ok(hash)
    }
}

/// An item within a `Tree`, this could be a file blob or another directory.
#[derive(Debug)]
enum TreeItem<'a> {
    /// Refers to a file by hash
    Blob(HashOutput),
    /// Refers to a nested directory
    Tree(Tree<'a>),
}
