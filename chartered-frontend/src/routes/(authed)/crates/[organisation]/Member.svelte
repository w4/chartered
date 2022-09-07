<script type="typescript">
    import Icon from '../../../../components/Icon.svelte';
    import { isEqual } from 'lodash';
    import Spinner from '../../../../components/Spinner.svelte';
    import { auth, BASE_URL } from '../../../../stores/auth';
    import ErrorAlert from '../../../../components/ErrorAlert.svelte';
    import { createEventDispatcher } from 'svelte';
    import type { CrateMember } from '../../../../types/crate';
    import { getErrorMessage } from '../../../../util';

    const dispatch = createEventDispatcher();

    /**
     * The name of the organisation that this is a `member` of.
     */
    export let organisation: string;

    /**
     * The name of the crate that this is a `member` of, or `null` if we're showing organisation
     * results.
     */
    export let crate: string | null = null;

    /**
     * The member to show
     */
    export let member: CrateMember;

    /**
     * A list of new permissions for the user, this is normally set by the user in the UI via bindings and default
     * to the user's current permissions. Whenever this differs from the user's current permissions then the save
     * icon is shown. This is exposed to the consumer for new ("prospective") members that don't currently exist
     * on the backend, where the consumer wants to give a default `VISIBLE` permission but also show the save icon.
     */
    export let newPermissions = member.permissions;

    /**
     * A list of possible permissions this user can be given.
     */
    export let possiblePermissions: string[];

    /**
     * A list of CSS classes to add to the outer div.
     */
    let clazz = '';
    export { clazz as class };

    /**
     * Whether the member is currently being persisted to the backend and a spinner is showing.
     */
    let saving = false;

    /**
     * Any errors that happened upon the last invocation of `save` to give feedback to the user.
     */
    let error: string | null = null;

    /**
     * Persist updates to this member to the backend.
     */
    async function save() {
        saving = true;
        error = null;

        try {
            // determine the HTTP verb to send for this membership change.
            let method;
            if (!newPermissions.includes('VISIBLE')) {
                // if the user is removing the VISIBLE permission from this member then it's a DELETE
                // operation otherwise their membership would be useless.
                method = 'DELETE';
            } else if (member.permissions.length === 0) {
                // if the member did not have initial permissions on this crate/org then they're a new
                // member to it, welcome aboard!
                method = 'PUT';
            } else {
                // anything else is simply just an update to an existing member
                method = 'PATCH';
            }

            // this component is called from both organisation views and crate views, so we need to figure
            // out which one we need to persist the changes to...
            let url;
            if (crate) {
                url = `web/v1/crates/${organisation}/${crate}`;
            } else {
                url = `web/v1/organisations/${organisation}`;
            }

            // send the membership update to the backend
            let result = await fetch(`${BASE_URL}/a/${$auth?.auth_key}/${url}/members`, {
                method,
                headers: {
                    Accept: 'application/json',
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({
                    user_uuid: member.uuid,
                    permissions: newPermissions,
                }),
            });

            let json = await result.json();

            if (json.error) {
                throw new Error(json.error);
            }

            // fast-update the permissions locally to hide the save button, then prompt the parent
            // component to update their membership list so the user gets the most up-to-date view
            // of permissions that the server sees.
            member.permissions = newPermissions;
            dispatch('updated', member.uuid);
        } catch (e) {
            error = getErrorMessage(e);
        } finally {
            saving = false;
        }
    }
</script>

<div class="p-6 {clazz}">
    {#if error}
        <ErrorAlert on:close={() => (error = null)}>{error}</ErrorAlert>
    {/if}

    <div class="flex flex-col md:flex-row md:items-center">
        <a href={`/users/${member.uuid}`} class="flex-grow flex items-center mr-10 card-header mb-2 md:mb-0">
            {#if member.picture_url}
                <img
                    alt={member.display_name}
                    src={member.picture_url}
                    class="rounded-[50%] h-[2rem] mr-3 inline-block"
                />
            {:else}
                <div
                    class="inline-block h-[2rem] w-[2rem] rounded-[50%] mr-3 text-gray-300 bg-gray-100 dark:bg-gray-900 overflow-hidden"
                >
                    <Icon height="2rem" width="2rem" name="user" />
                </div>
            {/if}

            {member.display_name}
        </a>

        <div>
            {#each possiblePermissions as permission}
                <div class="flex md:inline-flex items-center md:mr-4">
                    <input
                        id={`${member.uuid}-${permission}`}
                        bind:group={newPermissions}
                        value={permission}
                        type="checkbox"
                        class="w-4 h-4 mr-2 rounded border border-gray-200 dark:border-gray-700 bg-transparent ring-blue-500 focus:border-blue-500 !ring-offset-0"
                    />
                    <label for={`${member.uuid}-${permission}`}>{permission}</label>
                </div>
            {/each}
        </div>

        <div
            class="flex items-center md:w-4 relative"
            class:hide={isEqual(newPermissions.sort(), member.permissions.sort())}
        >
            {#if saving}
                <div class="relative h-4 w-4 mt-2 md:mt-0">
                    <Spinner />
                </div>
            {:else if newPermissions.includes('VISIBLE')}
                <button
                    on:click={save}
                    class="text-lg btn-blue md:text-blue-700 md:p-0 md:border-none md:bg-transparent mt-2 md:mt-0 flex items-center"
                >
                    <Icon name="save" strokeWidth="2" />
                    <span class="ml-2 md:hidden">Save</span>
                </button>
            {:else}
                <button
                    on:click={save}
                    class="text-lg btn-red md:text-red-700 md:p-0 md:border-none md:bg-transparent mt-2 md:mt-0 flex items-center"
                >
                    <Icon name="trash" strokeWidth="2" />
                    <span class="ml-2 md:hidden">Delete</span>
                </button>
            {/if}
        </div>
    </div>
</div>

<style lang="postcss">
    .hide {
        @apply hidden md:block invisible;
    }
</style>
