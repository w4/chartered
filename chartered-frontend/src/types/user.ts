export interface User {
    uuid: string;
    username: string;
    name?: string;
    nick?: string;
    email?: string;
    external_profile_url?: string;
    picture_url?: string;
}

export interface UserSearch {
    users: UserSearchUser[];
}

export interface UserSearchUser {
    user_uuid: string;
    display_name: string;
    picture_url: string;
}
