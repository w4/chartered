<script type="typescript">
    import { page } from '$app/stores';
    import { request } from '../../../../../stores/auth';
    import Spinner from '../../../../../components/Spinner.svelte';
    import SvelteMarkdown from 'svelte-markdown';
    import Icon from '../../../../../components/Icon.svelte';
    import type { Crate } from '../../../../../types/crate';
    import Dependency from './Dependency.svelte';
    import VersionTab from './VersionTab.svelte';
    import MemberTab from './MemberTab.svelte';

    /**
     * Contains all the possible tabs, used for maintaining state on the current tab.
     */
    enum Tab {
        README,
        VERSIONS,
        MEMBERS,
    }

    /**
     * Mapping of `Tab`s to their human-readable form alongside a friendly icon to show to the
     * user.
     */
    const allTabs = [
        {
            id: Tab.README,
            name: 'Readme',
            icon: 'book-open',
        },
        {
            id: Tab.VERSIONS,
            name: 'Versions',
            icon: 'archive',
        },
        {
            id: Tab.MEMBERS,
            name: 'Members',
            icon: 'user',
        },
    ];

    // lookup the crate currently requested by the user based on the URL
    let cratePromise: Promise<Crate>;
    $: cratePromise = request(`/web/v1/crates/${$page.params.organisation}/${$page.params.crate}`);

    // binding to the current tab the user has selected
    let currentTab = Tab.README;
</script>

<header>
    <div class="container flex mx-auto p-10 mb-3">
        <div class="flex-grow">
            <h1 class="text-5xl font-bold tracking-tight">
                <a href={`/crates/${$page.params.organisation}`}>{$page.params.organisation}</a>/<span
                    class="text-highlight">{$page.params.crate}</span
                >
            </h1>

            {#await cratePromise then crate}
                <h2>
                    {#if crate.description}
                        {crate.description}
                    {:else}
                        <em>No description given.</em>
                    {/if}
                </h2>

                <div class="space-x-2">
                    {#if crate.homepage}
                        <a href={crate.homepage} target="_blank" class="card-header-button btn-blue-outline">
                            <Icon name="home" />
                            Home
                        </a>
                    {/if}

                    {#if crate.repository}
                        <a href={crate.repository} target="_blank" class="card-header-button btn-blue-outline">
                            <Icon name="git-branch" />
                            Repo
                        </a>
                    {/if}

                    {#if crate.documentation}
                        <a href={crate.documentation} target="_blank" class="card-header-button btn-blue-outline">
                            <Icon name="book" />
                            Docs
                        </a>
                    {/if}
                </div>
            {/await}
        </div>
    </div>
</header>

<main class="container mx-auto p-10 pt-0 grid grid-cols-12 gap-6 relative">
    {#await cratePromise}
        <Spinner />
    {:then crate}
        <div class="card col-span-full lg:col-span-9 p-0">
            <div class="border-b border-gray-200 dark:border-gray-700">
                <ul class="flex flex-wrap -mb-px text-sm font-medium text-center text-gray-500 dark:text-gray-400">
                    {#each allTabs as tab}
                        <li class="mr-2">
                            <button
                                on:click={() => (currentTab = tab.id)}
                                class:!border-blue-500={currentTab === tab.id}
                                class:text-blue-500={currentTab === tab.id}
                                aria-current={currentTab === tab.id ? 'page' : false}
                                class="inline-flex items-center space-x-2 p-4 rounded-t-lg border-b-2 border-transparent"
                            >
                                <Icon name={tab.icon} /> <span>{tab.name}</span>
                            </button>
                        </li>
                    {/each}
                </ul>
            </div>

            {#if currentTab === Tab.README}
                <article
                    class="mt-8 px-6 pb-6 prose dark:prose-invert text-inherit max-w-full prose-headings:text-inherit hover:prose-a:text-blue-600 prose-a:text-blue-500 prose-code:p-1 prose-code:bg-slate-100 dark:prose-code:bg-slate-700 prose-code:rounded-lg prose-pre:bg-slate-100 dark:prose-pre:bg-slate-700 leading-6 before:prose-code:content-none after:prose-code:content-none prose-code:text-pink-400 prose-code:font-normal prose-strong:text-inherit prose-img:inline prose-img:my-0"
                >
                    <SvelteMarkdown source={crate.readme} />
                </article>
            {:else if currentTab === Tab.MEMBERS}
                <MemberTab />
            {:else if currentTab === Tab.VERSIONS}
                <div class="divide-y divide-gray-200 dark:divide-gray-700">
                    {#each crate.versions as version}
                        <VersionTab {version} class="p-6" />
                    {/each}
                </div>
            {/if}
        </div>

        <div class="col-span-full lg:col-span-3">
            <div class="card p-0">
                <h1 class="text-xl p-3 border-b border-gray-200 dark:border-gray-700 font-medium">Dependencies</h1>

                <div class="divide-y divide-gray-200 dark:divide-gray-700">
                    {#each crate.versions[0].deps as dependency}
                        <Dependency {dependency} class="p-3" />
                    {/each}
                </div>
            </div>
        </div>
    {/await}
</main>

<style lang="postcss">
    .card-header-button {
        @apply inline-flex items-center gap-x-1;
    }
</style>
