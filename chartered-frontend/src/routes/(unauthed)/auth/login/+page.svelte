<script type="typescript">
    import { loginOAuth, login, fetchOAuthProviders } from '../../../../stores/auth';
    import Spinner from '../../../../components/Spinner.svelte';
    import ErrorAlert from '../../../../components/ErrorAlert.svelte';
    import { goto } from '$app/navigation';

    /**
     * Once OAuth providers are loaded this will contain whether password auth
     * is enabled, we'll set it to true and just assume for now that password
     * auth is enabled so the user isn't sat waiting for a spinner.
     */
    let passwordAllowed = true;

    /**
     * Displays a spinner if a login is currently in progress, so we look busy
     * and the user can't modify form fields or anything.
     */
    let loginInProgress = false;

    /**
     * Displays an error to the user, if empty will hide the error popup.
     */
    let error = null;

    /**
     * A binding to the username field in the form.
     */
    let username = '';

    /**
     * A binding to the password field in the form.
     */
    let password = '';

    /**
     * Performs a password-based authentication using the bound username and password.
     */
    async function doLogin() {
        // start the spinner while the user is authenticating
        loginInProgress = true;

        try {
            await login(username, password);
        } catch (e) {
            error = e.toString();
        } finally {
            // behave like a real application and set the password to empty so
            // if the user fails authentication they can type it in again
            password = '';

            // stop the spinner now we've got our result
            loginInProgress = false;
        }
    }

    /**
     * Starts the OAuth flow for the given provider, grabbing the auth URL from the
     * backend.
     *
     * @param provider provider to start flow for
     */
    async function doLoginOAuth(provider: string) {
        // start the spinner while the user is authenticating
        loginInProgress = true;

        // reset both the username & password since the user attempted to do
        // an oauth login
        username = '';
        password = '';

        try {
            // attempt to redirect the user to the OAuth login page
            await loginOAuth(provider);
        } catch (e) {
            error = e.toString();
        } finally {
            loginInProgress = false;
        }
    }

    // start loading possible oauth providers
    const oauthProvidersPromise = fetchOAuthProviders().then((v) => {
        passwordAllowed = v.password;
        return v;
    });
</script>

<Spinner hidden={!loginInProgress} />

<div class:invisible={loginInProgress}>
    {#if error}
        <ErrorAlert on:close={() => (error = null)}>{error}</ErrorAlert>
    {/if}

    <form class:hidden={!passwordAllowed} on:submit|preventDefault={doLogin}>
        <div class="relative">
            <input type="text" id="username" class="peer" placeholder=" " bind:value={username} />
            <label for="username">Username</label>
        </div>

        <div class="relative">
            <input type="password" id="password" class="peer" placeholder=" " bind:value={password} />
            <label for="password">Password</label>
        </div>

        <button type="submit">Login</button>
    </form>

    <button class:hidden={!passwordAllowed} class="mt-2" on:click={() => goto('/auth/register')}> Register </button>

    {#await oauthProvidersPromise then oauthProviders}
        <div class:!hidden={!oauthProviders.password} class="side-lines">or</div>

        {#each oauthProviders.providers as provider}
            <button on:click={() => doLoginOAuth(provider)}>
                Login with {provider}
            </button>
        {/each}
    {/await}
</div>

<style>
    input {
        @apply w-full mb-2 px-2.5 pb-2.5 pt-5 bg-transparent border dark:border-slate-700 rounded border-inherit;
    }

    label {
        @apply absolute text-slate-500 duration-300 transform -translate-y-4 scale-75 top-4 z-10 origin-[0] left-2.5 peer-focus:text-blue-600 peer-focus:dark:text-blue-500 peer-placeholder-shown:scale-100 peer-placeholder-shown:translate-y-0 peer-focus:scale-75 peer-focus:-translate-y-4;
    }

    button {
        @apply text-white bg-blue-700 hover:bg-blue-800 focus:ring-4 focus:ring-blue-300 font-medium rounded-lg text-sm px-5 py-2.5 dark:bg-blue-600 dark:hover:bg-blue-700 focus:outline-none dark:focus:ring-blue-800 w-full;
    }

    .side-lines {
        @apply grid gap-3 text-sm text-center items-center grid-cols-sides-extend text-slate-500 my-2;
    }

    .side-lines::before,
    .side-lines::after {
        content: '';
        border-top: 1px solid;
    }
</style>
