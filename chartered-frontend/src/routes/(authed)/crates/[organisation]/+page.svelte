<script type="typescript">
    import { page } from '$app/stores';
    import { request } from '../../../../stores/auth';
    import Spinner from '../../../../components/Spinner.svelte';
    import ErrorAlert from '../../../../components/ErrorAlert.svelte';
    import Icon from '../../../../components/Icon.svelte';
    import type { OrganisationDetail } from '../../../../types/organisations';
    import Member from './Member.svelte';
    import AddMember from './AddMember.svelte';
    import type { CrateMembers, CrateMember } from '../../../../types/crate';

    // Load the requested organisation from the URL
    let organisationPromise: Promise<OrganisationDetail & CrateMembers>;
    $: organisationPromise = request(`/web/v1/organisations/${$page.params.organisation}`);

    /**
     * Whenever a member is updated/added/deleted to this organisation, we'll want to reload to ensure we
     * show the user exactly what the server currently sees.
     *
     * @param event a struct containing the updated member's UUID, so we can empty the newMember value if that member
     *              has just been added to we don't show them twice.
     */
    function reload(event: { detail: string }) {
        organisationPromise = request(`/web/v1/organisations/${$page.params.organisation}`);

        if (newMember && event.detail === newMember.uuid) {
            newMember = null;
        }
    }

    /**
     * Contains all the possible tabs, used for maintaining state on the current tab.
     */
    enum Tab {
        CRATES,
        MEMBERS,
    }

    /**
     * Mapping of `Tab`s to their human-readable form alongside a friendly icon to show to the
     * user.
     */
    const allTabs = [
        {
            id: Tab.CRATES,
            name: 'Crates',
            icon: 'package',
        },
        {
            id: Tab.MEMBERS,
            name: 'Members',
            icon: 'user',
        },
    ];

    // binding to the current tab the user has selected
    let currentTab = Tab.CRATES;

    // contains the member the user is currently considering adding to the org & has not yet persisted to
    // the server.
    let newMember: CrateMember | null = null;
</script>

<header>
    <div class="container flex flex-col md:flex-row items-center md:items-start mx-auto p-10 mb-3">
        <div class="flex-grow text-center md:text-left">
            <h1 class="text-5xl text-highlight font-bold tracking-tight">
                {$page.params.organisation}
            </h1>

            <h2>
                {#await organisationPromise}
                    <div class="relative h-4 w-4">
                        <Spinner />
                    </div>
                {:then organisation}
                    {#if organisation.description}
                        {organisation.description}
                    {:else}
                        <em>No description given.</em>
                    {/if}
                {/await}
            </h2>
        </div>

        <div class="order-first md:order-last">
            <img src="http://placekitten.com/128/128" alt="Placeholder" class="rounded-[50%]" />
        </div>
    </div>
</header>

<main class="container mx-auto p-10 pt-0">
    <div class="border-b border-gray-300 dark:border-gray-700">
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

    <div class="mt-4">
        {#await organisationPromise}
            <div class="relative h-14">
                <Spinner />
            </div>
        {:then organisation}
            {#if currentTab === Tab.CRATES}
                <div class="grid gap-2 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4">
                    {#each organisation.crates as crate}
                        <a href={`/crates/${$page.params.organisation}/${crate.name}`} class="card">
                            <h5 class="card-header">
                                {$page.params.organisation}/<span class="text-highlight">{crate.name}</span>
                            </h5>

                            <p class="card-body">
                                {#if crate.description}
                                    {crate.description}
                                {:else}
                                    <em>No description given.</em>
                                {/if}
                            </p>
                        </a>
                    {/each}
                </div>

                {#if organisation.crates.length === 0}
                    <div
                        class="p-4 text-sm text-gray-700 bg-gray-100 rounded-lg dark:bg-gray-700 dark:text-gray-300"
                        role="alert"
                    >
                        <span class="font-medium">Welcome to your new organisation!</span>
                        It's a little bit lonely here right now, but you can solve that easily by
                        <button
                            on:click={() => (currentTab = Tab.MEMBERS)}
                            class="underline text-blue-600 hover:text-blue-700"
                        >
                            adding some team members
                        </button>
                        or
                        <a
                            href="https://book.chart.rs/"
                            target="_blank"
                            class="underline text-blue-600 hover:text-blue-700"
                        >
                            publishing your first crate</a
                        >!
                    </div>
                {/if}
            {:else if currentTab === Tab.MEMBERS}
                <div class="card p-0 divide-y card-divide">
                    {#each organisation.members as member}
                        <Member
                            {member}
                            organisation={$page.params.organisation}
                            possiblePermissions={organisation.possible_permissions}
                            on:updated={reload}
                        />
                    {/each}

                    {#if newMember}
                        <Member
                            member={newMember}
                            newPermissions={['VISIBLE']}
                            organisation={$page.params.organisation}
                            possiblePermissions={organisation.possible_permissions}
                            on:updated={reload}
                        />
                    {/if}
                </div>

                <div class="card mt-4">
                    <AddMember
                        hideUuids={organisation.members.map((v) => v.uuid)}
                        on:new={(member) => {
                            member.detail.permissions = [];
                            member.detail.uuid = member.detail.user_uuid;
                            newMember = member.detail;
                        }}
                    />
                </div>
            {/if}
        {:catch e}
            <ErrorAlert showClose={false}>{e}</ErrorAlert>
        {/await}
    </div>
</main>
