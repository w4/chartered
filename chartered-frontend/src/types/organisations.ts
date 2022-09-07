export interface OrganisationList {
    organisations: OrganisationListItem[];
}

export interface OrganisationListItem {
    name: string;
    description: string;
}

export interface OrganisationDetail {
    description: string;
    crates: OrganisationCrate[];
}

export interface OrganisationCrate {
    name: string;
    description: string;
}
