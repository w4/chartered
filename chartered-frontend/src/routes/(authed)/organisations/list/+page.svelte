<script type="typescript">
    import { request } from '../../../../stores/auth';
    import type { OrganisationList } from '../../../../types/organisations';
    import ErrorAlert from '../../../../components/ErrorAlert.svelte';

    // fetch a list of all the current user's organisations from the backend
    const organisationsPromise = request<OrganisationList>('/web/v1/organisations');
</script>

<header>
    <div class="container flex items-center mx-auto">
        <div class="p-10 mb-3">
            <h1 class="text-5xl font-bold tracking-tight">
                Your <span class="text-highlight">Organisations</span>.
            </h1>
            <h2>
                Organisations and permissions are the heart of Chartered. All crates belong to an Organisation and to
                download a crate a user must have the <code>VISIBLE</code> permission for it.
            </h2>

            <a href="https://book.chart.rs/" target="_blank" class="block btn-blue-outline"> Learn More </a>
        </div>
    </div>
</header>

<main class="container mx-auto p-10 pt-0">
    {#await organisationsPromise}
        <div class="mb-4 grid md:grid-cols-2 lg:grid-cols-4 gap-2">
            {#each [1, 2, 3] as _}
                <div class="card flex space-x-2 items-center">
                    <div class="flex-grow h-full">
                        <div class="card-header">
                            <div class="skeleton-highlight w-32" />
                        </div>

                        <div class="card-body">
                            <div class="skeleton inline-block mr-2 w-24" />
                            <div class="skeleton inline-block mr-2 w-8" />
                            <div class="skeleton inline-block mr-2 w-16" />
                            <div class="skeleton inline-block mr-2 w-12" />
                            <div class="skeleton inline-block mr-2 w-9" />
                            <div class="skeleton inline-block w-10" />
                        </div>
                    </div>

                    <div class="min-w-[48px]">
                        <img alt="Placeholder" class="rounded-[50%]" src="http://placekitten.com/48/48" />
                    </div>
                </div>
            {/each}
        </div>
    {:then organisations}
        <div
            class:hidden={organisations.organisations.length === 0}
            class="mb-4 grid md:grid-cols-2 lg:grid-cols-4 gap-2"
        >
            {#each organisations.organisations as organisation}
                <a class="card flex space-x-2 items-center" href={`/crates/${organisation.name}`}>
                    <div class="flex-grow h-full">
                        <h5 class="text-highlight card-header">{organisation.name}</h5>
                        <p class="card-body">
                            {#if organisation.description}
                                {organisation.description}
                            {:else}
                                <em>No description given.</em>
                            {/if}
                        </p>
                    </div>

                    <div class="min-w-[48px]">
                        <img alt="Placeholder" class="rounded-[50%]" src="http://placekitten.com/48/48" />
                    </div>
                </a>
            {/each}
        </div>

        {#if organisations.organisations.length === 0}
            <div class="mb4">
                You currently belong to no organisations, please create one or ask someone to add you to an existing
                one.
            </div>
        {/if}
    {:catch e}
        <ErrorAlert showClose={false}>{e}</ErrorAlert>
    {/await}

    <a href="/organisations/create" class="inline-flex items-center btn-blue-outline"> + Create </a>
</main>
