<script type="typescript">
    import { request } from '../../../../stores/auth';
    import Spinner from '../../../../components/Spinner.svelte';
    import type { Session, Sessions } from '../../../../types/sessions';
    import RelativeTime from '../../../../components/RelativeTime.svelte';
    import Icon from '../../../../components/Icon.svelte';
    import DeleteSessionModal from './DeleteSessionModal.svelte';

    /**
     * If not null, then the user is currently attempting to delete a session and a modal is being
     * shown for them to confirm.
     */
    let deleting: Session | null = null;

    /**
     * Grab the list of current sessions from the user from the backend.
     */
    let sessionPromise: Promise<Sessions> = request<Sessions>('/web/v1/sessions');

    /**
     * Reload all the user's current sessions whenever a session is deleted so the user gets
     * an up-to-date view of what the backend sees.
     */
    function reloadSessions() {
        sessionPromise = request<Sessions>('/web/v1/sessions');
    }
</script>

<header>
    <div class="container flex items-center mx-auto">
        <div class="p-10 mb-3">
            <h1 class="text-5xl font-bold tracking-tight">Active Sessions</h1>
        </div>
    </div>
</header>

<main class="container mx-auto px-10">
    <div class="card">
        {#await sessionPromise}
            <div class="relative h-12">
                <Spinner />
            </div>
        {:then sessions}
            <div class="overflow-x-auto w-full">
                <table class="w-full max-w-full border-collapse">
                    <thead>
                        <tr>
                            <th scope="col">IP Address</th>
                            <th scope="col">User Agent</th>
                            <th scope="col">SSH Key Fingerprint</th>
                            <th scope="col">Expires</th>
                            <th scope="col" />
                        </tr>
                    </thead>

                    <tbody>
                        {#each sessions.sessions as session}
                            <tr>
                                <td>{session.ip}</td>
                                <td>{session.user_agent || 'n/a'}</td>
                                <td>{session.ssh_key_fingerprint || 'n/a'}</td>
                                <td>
                                    {#if session.expires_at}
                                        <RelativeTime time={session.expires_at} />
                                    {:else}
                                        n/a
                                    {/if}
                                </td>
                                <td>
                                    <button class="text-red-700" on:click={() => (deleting = session)}>
                                        <Icon name="trash" />
                                    </button>
                                </td>
                            </tr>
                        {/each}
                    </tbody>
                </table>
            </div>
        {/await}
    </div>
</main>

{#if deleting}
    <DeleteSessionModal {deleting} on:complete={reloadSessions} on:close={() => (deleting = null)} />
{/if}

<style lang="postcss">
    th,
    td {
        @apply text-left px-5;
    }

    thead tr:first-of-type th {
        @apply pb-2;
    }

    tbody tr td {
        @apply py-2;
    }

    tbody tr:last-of-type td {
        @apply pb-0;
    }
</style>
