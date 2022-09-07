<script type="typescript">
    import { auth, logout } from '../stores/auth';
    import Icon from './Icon.svelte';
    import { goto } from '$app/navigation';

    /**
     * Nav drawer is hidden by default on screen sizes < md, if set to true the nav drawer will
     * be showing on those screen sizes.
     */
    let navDrawerShown = false;

    /**
     * If set to true, the "user menu" dropdown will be showing.
     */
    let userDropdownShown = false;

    /**
     * Binding to the search field.
     */
    let search = '';

    /**
     * Performs a search with the current value of `search`.
     */
    async function performSearch() {
        await goto(`/search?q=${search}`);
    }

    /**
     * Prevents the dropdown from automatically closing if it was a non-link element
     * that was clicked.
     *
     * @param e mouse click event
     */
    function handleDropdownClick(e: MouseEvent) {
        let tagName = (e.target as HTMLElement)?.tagName.toLowerCase();

        if (tagName !== 'a' && tagName != 'button') {
            e.stopPropagation();
        }
    }
</script>

<svelte:window on:click={() => (userDropdownShown = false)} />

<nav class="bg-white border-gray-200 px-2 sm:px-4 py-2.5 rounded dark:bg-gray-900">
    <div class="container flex flex-wrap items-center mx-auto">
        <a href="/" class="flex items-center flex-grow md:flex-grow-0 md:mr-3">
            <span class="self-center text-xl font-semibold whitespace-nowrap dark:text-white">
                ✈️&nbsp;&nbsp;Chartered
            </span>
        </a>

        <div class="flex md:order-2">
            <form on:submit|preventDefault={performSearch} class="hidden relative md:block">
                <div class="flex absolute inset-y-0 left-0 items-center pl-3 pointer-events-none text-gray-500">
                    <span class="sr-only">Search icon</span>
                    <Icon name="search" />
                </div>

                <input
                    bind:value={search}
                    type="search"
                    class="block p-2 pl-10 w-full text-gray-900 bg-gray-50 rounded-lg border border-gray-300 sm:text-sm focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500"
                    placeholder="Search..."
                />
            </form>

            <button
                type="button"
                class="inline-flex items-center p-2 text-sm text-gray-500 rounded-lg md:hidden hover:bg-gray-100 focus:outline-none focus:ring-2 focus:ring-gray-200 dark:text-gray-400 dark:hover:bg-gray-700 dark:focus:ring-gray-600"
                on:click={() => (navDrawerShown = !navDrawerShown)}
                aria-expanded={navDrawerShown}
            >
                <span class="sr-only">Open menu</span>
                <Icon height="1.5rem" width="1.5rem" strokeWidth="2" name="menu" />
            </button>

            <div class="relative">
                <button
                    on:click|stopPropagation={() => (userDropdownShown = !userDropdownShown)}
                    class="ml-2 relative flex items-center text-gray-300 dark:text-inherit"
                >
                    <span class="sr-only">Open user menu</span>
                    {#if $auth?.picture_url}
                        <img alt="You" src={$auth?.picture_url} class="rounded-[50%] h-[2.4rem] inline mr-0.5" />
                    {:else}
                        <div
                            class="h-[2.4rem] w-[2.4rem] rounded-[50%] inline mr-0.5 bg-gray-100 dark:bg-gray-800 overflow-hidden"
                        >
                            <Icon height="2.4rem" width="2.4rem" name="user" />
                        </div>
                    {/if}
                    <Icon name="chevron-down" strokeWidth="3px" />
                </button>

                <div
                    class:hidden={!userDropdownShown}
                    class="absolute top-7 right-0 z-50 w-[10rem] my-4 text-base list-none bg-white rounded divide-y divide-gray-100 shadow dark:bg-gray-700 dark:divide-gray-600"
                    on:click={handleDropdownClick}
                >
                    <ul class="py-1">
                        <li>
                            <a
                                href={`/users/${$auth?.uuid}`}
                                class="block py-2 px-4 text-sm text-gray-700 hover:bg-gray-100 dark:hover:bg-gray-600 dark:text-gray-200 dark:hover:text-white"
                            >
                                Profile
                            </a>
                        </li>
                    </ul>

                    <ul class="py-1">
                        <li>
                            <a
                                href="/sessions/list"
                                class="block py-2 px-4 text-sm text-gray-700 hover:bg-gray-100 dark:hover:bg-gray-600 dark:text-gray-200 dark:hover:text-white"
                            >
                                Active Sessions
                            </a>
                        </li>
                        <li>
                            <button
                                on:click={logout}
                                class="block py-2 px-4 w-full text-start text-sm text-gray-700 hover:bg-gray-100 dark:hover:bg-gray-600 dark:text-gray-200 dark:hover:text-white"
                            >
                                Sign out
                            </button>
                        </li>
                    </ul>
                </div>
            </div>
        </div>

        <div
            class:hidden={!navDrawerShown}
            class="justify-between items-center w-full md:flex md:w-auto md:flex-grow md:order-1"
            id="navbar-search"
        >
            <form on:submit|preventDefault={performSearch} class="relative mt-3 md:hidden">
                <div class="flex absolute inset-y-0 left-0 items-center pl-3 pointer-events-none text-gray-500">
                    <Icon name="search" />
                </div>

                <input
                    bind:value={search}
                    type="search"
                    class="block p-2 pl-10 w-full text-gray-900 bg-gray-50 rounded-lg border border-gray-300 sm:text-sm focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500"
                    placeholder="Search..."
                />
            </form>

            <ul
                class="flex flex-col p-4 mt-4 bg-gray-50 rounded-lg border border-gray-100 md:flex-row md:space-x-8 md:mt-0 md:text-sm md:font-medium md:border-0 md:bg-white dark:bg-gray-800 md:dark:bg-gray-900 dark:border-gray-700"
            >
                <slot />
            </ul>
        </div>
    </div>
</nav>
