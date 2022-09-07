<script type="typescript">
    import Spinner from '../../../../components/Spinner.svelte';
    import ErrorAlert from '../../../../components/ErrorAlert.svelte';
    import { register } from '../../../../stores/auth';
    import { goto } from '$app/navigation';

    /**
     * Displays a spinner while registration is being performed.
     */
    let registerInProgress = false;

    /**
     * Displays an error message to the user above the form if set to a string
     */
    let error = null;

    /**
     * The username bound to the corresponding form field
     */
    let username = '';

    /**
     * The password bound to the corresponding form field
     */
    let password = '';

    /**
     * Attempt the registration using the credentials currently in the form, setting
     * an error if the registration failed, or redirecting back to login form if it
     * was successful.
     */
    async function doRegister() {
        try {
            // start the spinner while we attempt to register the user
            registerInProgress = true;

            // attempt the registration
            await register(username, password);

            // if we got to this point registration was successful, so we don't need the
            // username to be set in the form anymore
            username = '';

            // redirect the user back to the login
            await goto('/auth/login');
        } catch (e) {
            error = e.toString();
        } finally {
            // stop displaying the spinner since we've finished attempting to register the
            // user
            registerInProgress = false;

            // unset the password so the user has to type it again like a real application
            password = '';
        }
    }
</script>

<Spinner hidden={!registerInProgress} />

<div class:invisible={registerInProgress}>
    {#if error}
        <ErrorAlert on:close={() => (error = null)}>{error}</ErrorAlert>
    {/if}

    <form on:submit|preventDefault={doRegister}>
        <div class="relative">
            <input type="text" id="username" class="peer" placeholder=" " bind:value={username} />
            <label for="username">Username</label>
        </div>

        <div class="relative">
            <input type="password" id="password" class="peer" placeholder=" " bind:value={password} />
            <label for="password">Password</label>
        </div>

        <button type="submit">Register</button>
    </form>
</div>

<style lang="postcss">
    input {
        @apply w-full mb-2 px-2.5 pb-2.5 pt-5 bg-transparent border dark:border-slate-700 rounded border-inherit;
    }

    label {
        @apply absolute text-slate-500 duration-300 transform -translate-y-4 scale-75 top-4 z-10 origin-[0] left-2.5 peer-focus:text-blue-600 peer-focus:dark:text-blue-500 peer-placeholder-shown:scale-100 peer-placeholder-shown:translate-y-0 peer-focus:scale-75 peer-focus:-translate-y-4;
    }

    button {
        @apply text-white bg-blue-700 hover:bg-blue-800 focus:ring-4 focus:ring-blue-300 font-medium rounded-lg text-sm px-5 py-2.5 dark:bg-blue-600 dark:hover:bg-blue-700 focus:outline-none dark:focus:ring-blue-800 w-full;
    }
</style>
