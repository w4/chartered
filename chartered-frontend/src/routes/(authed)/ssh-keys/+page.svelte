<script type="typescript">
    import { request } from '../../../stores/auth';
    import Spinner from '../../../components/Spinner.svelte';
    import ErrorAlert from '../../../components/ErrorAlert.svelte';
    import SingleSshKey from './SingleSshKey.svelte';
    import type { SshKeys, SshKey } from '../../../types/ssh_keys';
    import CreateKeyForm from './CreateKeyForm.svelte';
    import DeleteSshKeyModal from './DeleteSshKeyModal.svelte';

    // contains the key that is currently being considered by the user for deletion,
    // if this is set then the user is currently seeing a popup for confirmation
    let deleting: SshKey | null = null;

    // loads all the user's SSH keys from the backend
    let sshKeysPromise = fetchSshKeys();

    /**
     * Fetches the current user's SSH keys from the backend
     */
    function fetchSshKeys(): Promise<SshKeys> {
        return request<SshKeys>('/web/v1/ssh-key');
    }

    /**
     * Reloads the user's SSH keys from the backend, this will cause a spinner while they
     * are reloaded
     */
    function reloadSshKeys() {
        sshKeysPromise = fetchSshKeys();
    }
</script>

<header>
    <div class="container flex items-center mx-auto">
        <div class="p-10 mb-3">
            <h1 class="text-5xl font-bold tracking-tight">
                Manage your <span class="text-highlight">SSH Keys</span>.
            </h1>
            <h2>SSH keys are how Chartered identifies your account when called via Cargo.</h2>

            <a href="https://book.chart.rs/" target="_blank" class="block btn-blue-outline"> Learn More </a>
        </div>
    </div>
</header>

<main class="container mx-auto p-10 pt-0">
    {#await sshKeysPromise}
        <div class="relative h-4">
            <Spinner />
        </div>
    {:then sshKeys}
        <div class:hidden={sshKeys.keys.length === 0} class="card mb-4 p-2">
            <div class="overflow-scroll">
                <div class="w-fit min-w-full">
                    {#each sshKeys.keys as sshKey, i}
                        {#if i > 0}<hr class="card-hr" />{/if}

                        <SingleSshKey {sshKey} on:delete={() => (deleting = sshKey)} />
                    {/each}
                </div>
            </div>
        </div>

        <CreateKeyForm on:complete={reloadSshKeys} />
    {:catch e}
        <ErrorAlert showClose={false}>{e}</ErrorAlert>
    {/await}
</main>

{#if deleting}
    <DeleteSshKeyModal {deleting} on:complete={reloadSshKeys} on:close={() => (deleting = null)} />
{/if}
