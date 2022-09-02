export interface Sessions {
    sessions: Session[];
}

export interface Session {
    uuid: string;
    expires_at?: string;
    user_agent?: string;
    ip: string;
    ssh_key_fingerprint?: string;
}
