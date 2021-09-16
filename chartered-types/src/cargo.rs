//! 'Raw' types that are passed by `cargo publish` and also consumed via
//! cargo when pulling. These are just inserted into the database as-is.

use serde::{Deserialize, Serialize};
use std::{borrow::Cow, collections::BTreeMap};

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct CrateVersion<'a> {
    #[serde(borrow)]
    pub name: Cow<'a, str>,
    #[serde(borrow)]
    pub vers: Cow<'a, str>,
    pub deps: Vec<CrateDependency<'a>>,
    pub features: CrateFeatures,
    #[serde(borrow)]
    pub links: Option<Cow<'a, str>>,
}

impl CrateVersion<'_> {
    pub fn into_owned(self) -> CrateVersion<'static> {
        CrateVersion {
            name: Cow::Owned(self.name.into_owned()),
            vers: Cow::Owned(self.vers.into_owned()),
            deps: self.deps.into_iter().map(|v| v.into_owned()).collect(),
            features: self.features,
            links: self.links.map(|v| Cow::Owned(v.into_owned())),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct CrateVersionMetadata {
    pub description: Option<String>,
    pub readme: Option<String>,
    pub repository: Option<String>,
    pub homepage: Option<String>,
    pub documentation: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct CrateDependency<'a> {
    pub name: Cow<'a, str>,
    pub version_req: Cow<'a, str>, // needs to be: https://github.com/steveklabnik/semver#requirements
    pub features: Vec<Cow<'a, str>>,
    pub optional: bool,
    pub default_features: bool,
    pub target: Option<Cow<'a, str>>, // a string such as "cfg(windows)"
    pub kind: Cow<'a, str>,           // dev, build or normal
    pub registry: Option<Cow<'a, str>>,
    pub package: Option<Cow<'a, str>>,
}

impl CrateDependency<'_> {
    pub fn into_owned(self) -> CrateDependency<'static> {
        CrateDependency {
            name: Cow::Owned(self.name.into_owned()),
            version_req: Cow::Owned(self.version_req.into_owned()),
            features: self
                .features
                .into_iter()
                .map(|v| Cow::Owned(v.into_owned()))
                .collect(),
            optional: self.optional,
            default_features: self.default_features,
            target: self.target.map(|v| Cow::Owned(v.into_owned())),
            kind: Cow::Owned(self.kind.into_owned()),
            registry: self.registry.map(|v| Cow::Owned(v.into_owned())),
            package: self.package.map(|v| Cow::Owned(v.into_owned())),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct CrateFeatures(pub BTreeMap<String, Vec<String>>);
