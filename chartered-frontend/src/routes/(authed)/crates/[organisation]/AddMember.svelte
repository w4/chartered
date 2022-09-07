<script type="typescript">
    import { debounce } from 'lodash';
    import { request } from '../../../../stores/auth';
    import Spinner from '../../../../components/Spinner.svelte';
    import Icon from '../../../../components/Icon.svelte';
    import { createEventDispatcher } from 'svelte';
    import type { UserSearch, UserSearchUser } from '../../../../types/user';

    // Create the dispatcher to send an event whenever a new member is selected
    // by the user.
    const dispatch = createEventDispatcher();

    /**
     * A list of UUIDs to hide from the search, so we can skip showing users that
     * are already members.
     */
    export let hideUuids: string[] = [];

    /**
     * Contains whether the search results are currently loading so a spinner
     * can be shown.
     */
    let loading = false;

    /**
     * Binding to the search terms the user has entered.
     */
    let search = '';

    /**
     * A list of search results from the backend
     */
    let searchResults: UserSearchUser[] = [];

    // update `searchResults` whenever `search` is updated
    $: performSearch(search);

    // debounce the user's input for 250ms, so we don't just spam the backend with search
    // requests even though the user isn't finished yet.
    const onInput = debounce((event) => {
        search = event.target.value;
    }, 250);

    /**
     * Call the backend and fetch user results for the user's given search terms.
     *
     * @param search terms to search for
     */
    async function performSearch(search: string) {
        if (search === '') {
            return;
        }

        loading = true;

        try {
            let result = await request<UserSearch>(`/web/v1/users/search?q=${search}`);
            searchResults = result.users || [];
        } catch (e: unknown) {
            console.log(e);
        } finally {
            loading = false;
        }
    }

    /**
     * Send an event back to the parent component whenever a user is selected.
     *
     * @param member member to send to the parent component
     */
    function dispatchNewMember(member: UserSearchUser) {
        dispatch('new', member);
        searchResults = [];
        search = '';
    }
</script>

<div class="flex items-center">
    <input
        placeholder="Start typing a username..."
        on:input={(e) => {
            searchResults = [];
            onInput(e);
        }}
        type="text"
        class="border border-gray-200 dark:border-gray-700 bg-transparent text-sm rounded-lg block p-2.5 ring-blue-500 w-[100%] lg:w-[25%] focus:border-blue-500 mr-1"
    />

    {#if loading}
        <div class="relative h-10 w-10">
            <Spinner />
        </div>
    {/if}
</div>

{#if searchResults.length !== 0}
    <div class="mt-4 grid md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
        {#each searchResults as result}
            {#if !hideUuids.includes(result.user_uuid)}
                <button on:click={() => dispatchNewMember(result)} class="flex items-center">
                    {#if result.picture_url}
                        <img
                            alt={result.display_name}
                            src={result.picture_url}
                            class="rounded-[50%] h-[4rem] w-[4rem] mr-4"
                        />
                    {:else}
                        <div
                            class="h-[4rem] w-[4rem] rounded-[50%] text-gray-300 bg-gray-100 dark:bg-gray-800 overflow-hidden mr-4"
                        >
                            <Icon height="4rem" width="4rem" name="user" />
                        </div>
                    {/if}

                    <span class="card-header">{result.display_name}</span>
                </button>
            {/if}
        {/each}
    </div>
{/if}
