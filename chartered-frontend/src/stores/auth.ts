import { get, writable } from 'svelte/store';
import { goto } from '$app/navigation';

/**
 * The base URL of the chartered-web instance
 */
export const BASE_URL = import.meta.env.CHARTERED_WEB_URL || 'http://127.0.0.1:8888';

/**
 * Key in localStorage to store authentication information in
 */
const AUTH_LOCAL_STORAGE_KEY = 'auth.auth';

/**
 * Structure of the authentication information in local storage
 */
interface Auth {
    uuid: string;
    auth_key: string;
    expires: number;
    picture_url: string;
}

// grab the initial state from localStorage and initialise the actual store with the value we
// grabbed, if there was one.
const initialState = window.localStorage.getItem(AUTH_LOCAL_STORAGE_KEY);
export const auth = writable<Auth | null>(initialState ? JSON.parse(initialState) : null);

// subscribe to the auth store change events, so we can persist the changes to localStorage
auth.subscribe((v) => window.localStorage.setItem(AUTH_LOCAL_STORAGE_KEY, JSON.stringify(v)));

/**
 * Error response that all of our chartered-web endpoints conform to
 */
interface Error {
    error?: string;
}

/**
 * Response type of /web/v1/auth/extend, used to extend the user's current session
 */
interface ExtendResponse {
    expires: string;
}
type ExtendResult = ExtendResponse & Error;

/**
 * Attempt to extend the user's session, logging the user out if the session has already expired
 * before we were called.
 */
export async function extendSession() {
    // grab the current value of the auth store
    const currentAuth = get(auth);

    // sanity check to ensure that the user has a valid session according to what we have in our
    // store, logging the user out if our store knows the token is already expired
    if (currentAuth === null) {
        // the user has no active session, there's nothing we can do here
        return;
    } else if (currentAuth.expires < Date.now()) {
        // the store thinks that the token has expired, so it probably has. let's ensure the server
        // thinks the same by attempting a logout, and then we'll clear the session from the store
        await logout();
        return;
    }

    try {
        // call chartered-web to attempt to extend the session
        const result = await fetch(`${BASE_URL}/a/${currentAuth.auth_key}/web/v1/auth/extend`);
        const json: ExtendResult = await result.json();

        // backend returned an error, nothing we can do here
        if (json.error) {
            throw new Error(json.error);
        }

        // we got a successful response back from the server with the next expiry time so lets
        // update our local store
        currentAuth.expires = Date.parse(json.expires);
        auth.set(currentAuth);
    } catch (e) {
        // if the user's token is invalid for whatever reason after trying to refresh it,
        // we should log them out. otherwise, we don't really know how to handle the error,
        // and we should just print the error out to the console
        if (e === 'Expired auth token') {
            auth.set(null);
        } else {
            console.error('Failed to extend user session', e);
        }
    }
}

// start the cron loop to extend the session every minute
if (!window.extendSessionInterval) {
    extendSession();
    window.extendSessionInterval = setInterval(extendSession, 60000);
}

/**
 * Successful response type of /web/v1/auth/login/password, returned once the user has
 * an active session on the backend
 */
interface LoginResponse {
    user_uuid: string;
    key: string;
    expires: string;
    picture_url: string;
}
type LoginResult = LoginResponse & Error;

/**
 * Attempt to log the user in using password-based auth with the given credentials,
 * throwing an error if the credentials are invalid or another error occurred.
 *
 * @param username username to attempt to log in with
 * @param password password to attempt to log in with
 */
export async function login(username: string, password: string) {
    // call the backend and attempt the authentication
    const result = await fetch(`${BASE_URL}/a/-/web/v1/auth/login/password`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ username, password }),
    });
    const json: LoginResult = await result.json();

    // server returned an error, forward it on - there's nothing else we
    // can do here
    if (json.error) {
        throw new Error(json.error);
    }

    // we got a successful response back from the server, get in there son
    auth.set({
        auth_key: json.key,
        expires: Date.parse(json.expires),
        picture_url: json.picture_url,
        uuid: json.user_uuid,
    });
}

/**
 * Attempt to log the user in using the OAuth callback throwing an error if an error occurred.
 *
 * @param params URL search parameters
 */
export async function handleOAuthCallback(params: string) {
    // call the backend and attempt the authentication
    const result = await fetch(`${BASE_URL}/a/-/web/v1/auth/login/oauth/complete${params}`);
    const json: LoginResult = await result.json();

    // server returned an error, forward it on - there's nothing else we
    // can do here
    if (json.error) {
        throw new Error(json.error);
    }

    // we got a successful response back from the server, get in there son
    auth.set({
        auth_key: json.key,
        expires: Date.parse(json.expires),
        picture_url: json.picture_url,
        uuid: json.user_uuid,
    });
}

/**
 * Successful response type of /web/v1/auth/login/oauth/[provider]/begin, contains the URL
 * the user needs to visit to complete the OAuth flow.
 */
interface LoginOAuthResponse {
    redirect_url: string;
}
type LoginOAuthResult = LoginOAuthResponse & Error;

/**
 * Sends a GET request to the backend using the users current credentials.
 *
 * @param url url (without base) to send request to
 */
export async function request<T>(url: string): Promise<T> {
    const token = get(auth)?.auth_key;

    if (!token) {
        throw new Error('Not authenticated');
    }

    const result = await fetch(`${BASE_URL}/a/${token}${url}`);
    const json: T & Error = await result.json();

    // TODO: handle 404s
    if (json.error) {
        throw new Error(json.error);
    }

    return json;
}

/**
 * Grab an authentication URL for the provider and redirect the user to it.
 *
 * @param provider OAuth provider as configured on the backend to grab an auth link for
 */
export async function loginOAuth(provider: string) {
    const result = await fetch(`${BASE_URL}/a/-/web/v1/auth/login/oauth/${provider}/begin`);
    const json: LoginOAuthResult = await result.json();

    if (json.error) {
        throw new Error(json.error);
    }

    await goto(json.redirect_url);
}

/**
 * Send a request to the backend to clear this session, ignoring the response and clearing
 * our local store regardless of the result.
 *
 * If the session still happens to be active on the backend after this, the user can still
 * clear it out through the UI.
 */
export async function logout() {
    try {
        const authKey = get(auth)?.auth_key;

        if (authKey) {
            await fetch(`${BASE_URL}/a/${authKey}/web/v1/auth/logout`);
        }
    } catch (e) {
        console.error('Failed to fully log user out of session', e);
    } finally {
        auth.set(null);
    }
}

/**
 * A list of possible authentication methods for the user, returning OAuth providers and
 * whether password auth is enabled.
 */
interface OAuthProviders {
    password: boolean;
    providers: string[];
}

/**
 * Grab all the possible authentication methods from the backend.
 */
export function fetchOAuthProviders(): Promise<OAuthProviders> {
    return fetch(`${BASE_URL}/a/-/web/v1/auth/login/oauth/providers`).then((v) => v.json());
}

/**
 * Response of /web/v1/auth/register/password endpoint for password-based authentication.
 */
interface RegisterResponse {
    success: boolean;
}
type RegisterResult = RegisterResponse & Error;

/**
 * Attempt to register a user with the given credentials, throwing an error if registration
 * fails for whatever reason.
 *
 * @param username username to register
 * @param password password to register
 */
export async function register(username: string, password: string) {
    // send register request to backend
    const result = await fetch(`${BASE_URL}/a/-/web/v1/auth/register/password`, {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
        },
        body: JSON.stringify({ username, password }),
    });
    const json: RegisterResult = await result.json();

    // throw an error if registration fails
    if (json.error) {
        throw new Error(json.error);
    } else if (!json.success) {
        throw new Error('Failed to register, please try again later.');
    }
}
