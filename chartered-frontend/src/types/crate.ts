export interface Crate {
    name: string;
    readme?: string;
    description?: string;
    repository?: string;
    homepage?: string;
    documentation?: string;
    versions: Version[];
}

export interface Version {
    name: string;
    deps: VersionDependency[];
    vers: string;
    features: { [key: string]: string[] };
    size: number;
    created_at: string;
    uploader: VersionUploader;
}

export interface VersionDependency {
    name: string;
    req: string;
    registry?: string;
}

export interface VersionUploader {
    uuid: string;
    display_name: string;
    picture_url?: string;
}

export interface Search {
    crates: SearchCrate[];
}

export interface SearchCrate {
    organisation: string;
    name: string;
    description?: string;
    version: string;
    homepage?: string;
    repository?: string;
}

export interface CrateMembers {
    possible_permissions: string[];
    implied_permissions: [string[], string[]][];
    members: CrateMember[];
}

export interface CrateMember {
    uuid: string;
    display_name: string;
    picture_url?: string;
    permissions: string[];
}
