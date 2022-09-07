<script type="typescript">
    import { page } from '$app/stores';
    import { request } from '../../../stores/auth';
    import Spinner from '../../../components/Spinner.svelte';
    import type { Search } from '../../../types/crate';

    // whenever the `q` query parameter changes, send a request to the backend with that
    // search term.
    let searchPromise: Promise<Search>;
    $: searchPromise = request(`/web/v1/crates/search?q=${$page.url.searchParams.get('q')}`);
</script>

<header>
    <div class="container flex items-center mx-auto">
        <div class="p-10 mb-3">
            <h1 class="text-5xl font-bold tracking-tight">
                Search results for "<span class="text-highlight">{$page.url.searchParams.get('q')}</span>".
            </h1>
        </div>
    </div>
</header>

{#await searchPromise}
    <div class="relative h-12">
        <Spinner />
    </div>
{:then results}
    <main class="container mx-auto p-10 grid gap-2 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4">
        {#each results.crates as crate}
            <div class="card">
                <div class="card-header">
                    <a href={`/crates/${crate.organisation}/${crate.name}`}>
                        {crate.organisation}/<span class="text-highlight">{crate.name}</span>
                    </a>

                    <small class="text-sm text-gray-400 font-normal">{crate.version}</small>
                </div>

                <div class="card-body">
                    {crate.description}
                </div>

                <div class="text-sm mt-2 text-blue-600">
                    {#if crate.homepage}
                        <a href={crate.homepage} target="_blank" class="mr-2">Homepage</a>
                    {/if}

                    {#if crate.repository}
                        <a href={crate.repository} target="_blank">Repository</a>
                    {/if}
                </div>
            </div>
        {/each}
    </main>
{/await}
