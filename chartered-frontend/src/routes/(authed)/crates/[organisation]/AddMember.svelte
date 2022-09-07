<script type="typescript">
    import { debounce } from 'lodash';
    import { auth, BASE_URL } from '../../../../stores/auth';
    import Spinner from '../../../../components/Spinner.svelte';
    import Icon from '../../../../components/Icon.svelte';
    import { createEventDispatcher } from 'svelte';

    const dispatch = createEventDispatcher();

    export let hideUuids: string[] = [];

    let loading = false;

    let search = '';
    let searchResults = [];
    $: performSearch(search);

    const onInput = debounce((event) => {
        search = event.target.value;
    }, 250);

    async function performSearch(search: string) {
        if (search === '') {
            return;
        }

        loading = true;

        try {
            let result = await fetch(`${BASE_URL}/a/${$auth.auth_key}/web/v1/users/search?q=${search}`);
            let json = await result.json();

            if (json.error) {
                throw new Error(json.error);
            }

            searchResults = json.users || [];
        } catch (e) {
            console.log(e);
        } finally {
            loading = false;
        }
    }

    function dispatchNewMember(member) {
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
