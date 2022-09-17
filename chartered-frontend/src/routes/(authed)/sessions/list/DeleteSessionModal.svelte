<script type="typescript">
    import { createEventDispatcher } from 'svelte';
    import Icon from '../../../../components/Icon.svelte';
    import ErrorAlert from '../../../../components/ErrorAlert.svelte';
    import Spinner from '../../../../components/Spinner.svelte';
    import { auth, BASE_URL } from '../../../../stores/auth';
    import type { Session } from '../../../../types/sessions';
    import { getErrorMessage } from '../../../../util';

    const dispatch = createEventDispatcher();

    /**
     * The key that the user requested to delete.
     */
    export let deleting: Session;

    /**
     * Any errors that came of the last submission. If this is not null the user will be shown
     * an alert.
     */
    let deleteError: string | null = null;

    /**
     * Simple boolean flag to determine if a session is currently being submitted, so we can
     * disable the buttons and prevent exiting the modal.
     */
    let isDeleting = false;

    /**
     * Binds helper keys for the modal, so we can provide some expected functions of a modal.
     */
    function handleKeydown(event: KeyboardEvent) {
        if (event.key == 'Escape') {
            closeModal();
        }
    }

    /**
     * Attempts the deletion of the session against the backend.
     */
    async function deleteSession() {
        // reset the error since it no longer applies after this run
        deleteError = null;

        // show a spinner to the user while we're working and prevent them leaving the
        // modal
        isDeleting = true;

        try {
            // submit deletion request to backend
            let res = await fetch(`${BASE_URL}/web/v1/sessions`, {
                method: 'DELETE',
                headers: {
                    'Content-Type': 'application/json',
                    Authorization: `Bearer ${$auth?.auth_key}`,
                },
                body: JSON.stringify({ uuid: deleting.uuid }),
                credentials: 'include',
            });
            let json: { error?: string } = await res.json();

            if (json.error) {
                throw new Error(json.error);
            }

            // key successfully deleted, alert the main view, so it can reload the key list,
            // and close the modal
            isDeleting = false;
            dispatch('complete');
            closeModal();
        } catch (e) {
            isDeleting = false;
            deleteError = getErrorMessage(e);
        }
    }

    /**
     * Send an event to the main view to request to close us, by unsetting `deleting` on their end
     */
    function closeModal() {
        // if a deletion is currently in progress, we don't want to let the user close the modal
        // and just have to wonder whether the key was actually deleted
        if (!isDeleting) {
            dispatch('close');
        }
    }
</script>

<svelte:window on:keydown={handleKeydown} />

<div
    on:click|self={closeModal}
    class="overflow-y-auto overflow-x-hidden backdrop-blur-[4px] bg-black/20 h-screen w-screen fixed top-0 right-0 left-0 z-50 flex items-center justify-center"
>
    <div class="p-4 bg-white rounded-lg shadow-2xl dark:bg-gray-700">
        <button
            on:click={closeModal}
            type="button"
            class="absolute top-3 right-2.5 text-gray-400 bg-transparent hover:bg-gray-200 hover:text-gray-900 rounded-lg text-sm p-1.5 ml-auto inline-flex items-center dark:hover:bg-gray-800 dark:hover:text-white"
        >
            <Icon name="x" />
            <span class="sr-only">Close modal</span>
        </button>

        {#if deleteError}
            <ErrorAlert on:close={() => (deleteError = null)}>{deleteError}</ErrorAlert>
        {/if}

        <div class="p-6 text-center">
            <div class="mx-auto text-[4rem] text-red-600">
                <div class="inline-block">
                    <Icon name="alert-triangle" />
                </div>
            </div>

            <h3 class="text-lg font-normal text-inherit dark:text-gray-400">
                Are you sure you want to delete this session?
            </h3>

            <div class="break-all mb-5 mt-2 text-xs">
                <div><code>{deleting.ssh_key_fingerprint || deleting.user_agent || 'unknown'}</code></div>
                <div><code>{deleting.ip}</code></div>
            </div>

            {#if isDeleting}
                <div class="relative h-4">
                    <Spinner />
                </div>
            {:else}
                <button
                    on:click={deleteSession}
                    class="text-white bg-red-600 hover:bg-red-800 focus:ring-4 focus:outline-none focus:ring-red-300 dark:focus:ring-red-800 font-medium rounded-lg text-sm inline-flex items-center px-5 py-2.5 text-center mr-2"
                >
                    Yes, I'm sure
                </button>

                <!-- svelte-ignore a11y-autofocus -->
                <button
                    on:click={closeModal}
                    autofocus
                    class="text-gray-500 bg-white hover:bg-gray-100 focus:ring-4 focus:outline-none focus:ring-gray-200 rounded-lg border border-gray-200 text-sm font-medium px-5 py-2.5 hover:text-gray-900 focus:z-10 dark:bg-gray-700 dark:text-gray-300 dark:border-gray-500 dark:hover:text-white dark:hover:bg-gray-600 dark:focus:ring-gray-600"
                >
                    No, cancel
                </button>
            {/if}
        </div>
    </div>
</div>
