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

export interface OrganisationMember {
    uuid: string;
    display_name: string;
    picture_url?: string;
    permissions: string[];
}
