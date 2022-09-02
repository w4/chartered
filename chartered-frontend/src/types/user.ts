export interface User {
    uuid: string;
    username: string;
    name?: string;
    nick?: string;
    email?: string;
    external_profile_url?: string;
    picture_url?: string;
}
