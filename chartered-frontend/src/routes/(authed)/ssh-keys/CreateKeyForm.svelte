<script type="typescript">
    import Icon from '../../../components/Icon.svelte';
    import { auth, BASE_URL } from '../../../stores/auth';
    import type { AddSshKeyResult } from '../../../types/ssh_keys';
    import { createEventDispatcher } from 'svelte';
    import Spinner from '../../../components/Spinner.svelte';
    import ErrorAlert from '../../../components/ErrorAlert.svelte';

    // set up the event dispatcher for alerting the main page when the user has successfully
    // added a key, so it can reload the full key list from the backend again
    const dispatch = createEventDispatcher();

    /**
     * Binding to the ssh key text field, contains anything currently in there
     */
    let sshKey = '';

    /**
     * Simple boolean flag to determine if a key is currently being submitted, so we can
     * disable the form
     */
    let submitting = false;

    /**
     * Any errors that came of the last submission. If this is not null the user will be shown
     * an alert.
     */
    let error = null;

    /**
     * Submits an SSH key to the server to attempt to add it to the user, all our validation
     * will be done on the backend so this is just a simple fetch
     */
    async function submit() {
        // reset the error since it no longer applies after this submission
        error = null;

        // show a spinner to the user while we're working
        submitting = true;

        try {
            // submit the key to the backend
            let result = await fetch(`${BASE_URL}/a/${$auth.auth_key}/web/v1/ssh-key`, {
                method: 'PUT',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({ key: sshKey }),
            });
            let json: AddSshKeyResult = await result.json();

            if (json.error) {
                throw new Error(json.error);
            }

            // key was successfully added! reset the form and alert the main view, so
            // it can reload the key list
            sshKey = '';
            dispatch('complete');
        } catch (e) {
            error = e.toString();
        } finally {
            // stop showing the spinner since we've finished working
            submitting = false;
        }
    }
</script>

{#if error}
    <ErrorAlert on:close={() => (error = null)}>{error}</ErrorAlert>
{/if}

<form on:submit|preventDefault={submit}>
    <div class="card p-0">
        <div class="py-2 px-4 bg-white rounded-t-lg dark:bg-slate-800">
            <label for="key" class="sr-only">Your SSH key</label>
            <textarea
                id="key"
                bind:value={sshKey}
                disabled={submitting}
                rows="3"
                class="outline-none px-0 w-full text-sm text-gray-900 bg-white border-0 dark:bg-slate-800 focus:ring-0 dark:text-white dark:placeholder-gray-400"
                placeholder="ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAILYAIoV2OKRSh/DcM3TicD/NK/4TdqwwBPbKgFQKmGZ3 john@home"
                required
            />
        </div>

        <div class="flex justify-between items-center py-2 px-3 border-t border-gray-200 dark:border-gray-700">
            {#if submitting}
                <div class="relative h-4 w-4">
                    <Spinner />
                </div>
            {:else}
                <button
                    type="submit"
                    class="inline-flex items-center py-2.5 px-4 text-xs font-medium text-center text-white bg-blue-700 rounded-lg focus:ring-4 focus:ring-blue-200 dark:focus:ring-blue-900 hover:bg-blue-800"
                >
                    <Icon name="plus" />
                    <span class="ml-1">Add Key</span>
                </button>
            {/if}
        </div>
    </div>
</form>
