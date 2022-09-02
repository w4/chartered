export interface Crate {
    name: string;
    organisation: string;
}

export interface MostDownloaded {
    crates: Array<Crate & { downloads: number }>;
}

export interface RecentlyCreated {
    crates: Array<Crate & { created_at: string }>;
}

export interface RecentlyUpdated {
    versions: Array<Crate & { version: string }>;
}
