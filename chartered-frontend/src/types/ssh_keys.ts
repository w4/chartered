/**
 * The result of a `GET /web/v1/ssh-key`
 */
export interface SshKeys {
    keys: { [k: number]: SshKey };
}

/**
 * A singular SSH key from `SshKeys`.
 */
export interface SshKey {
    uuid: string;
    name: string;
    fingerprint: string;
    created_at: string;
    last_used_at?: string;
}

/**
 * The result of a `DELETE /web/v1/ssh-key`.
 */
export interface DeleteSshKeyResult {
    error?: string;
}

/**
 * The result of a `PUT /web/v1/ssh-key`.
 */
export interface AddSshKeyResult {
    error?: string;
}
