export interface OrganisationList {
    organisations: OrganisationListItem[];
}

export interface OrganisationListItem {
    name: string;
    description: string;
    public: boolean;
}

export interface OrganisationDetail {
    description: string;
    crates: OrganisationCrate[];
    public: boolean;
}

export interface OrganisationCrate {
    name: string;
    description: string;
}
