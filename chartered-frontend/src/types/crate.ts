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
    features: { [key: string]: any };
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
