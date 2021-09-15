import { AuthContext } from "./useAuth";

export const BASE_URL = 'http://localhost:8888';

export function unauthenticatedEndpoint(endpoint: string): string {
    return `${BASE_URL}/a/-/web/v1/${endpoint}`;
}

export function authenticatedEndpoint(auth: AuthContext, endpoint: string): string {
    return `${BASE_URL}/a/${auth.authKey}/web/v1/${endpoint}`;
}