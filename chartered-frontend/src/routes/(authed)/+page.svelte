<script type="typescript">
    import Icon from '../../components/Icon.svelte';
    import rustLogo from '../../img/rust.svg';
    import { request } from '../../stores/auth';
    import RelativeTime from '../../components/RelativeTime.svelte';
    import type { MostDownloaded, RecentlyCreated, RecentlyUpdated } from '../../types/featured_crate';
    import FeaturedCrate from '../../components/FeaturedCrate.svelte';
    import ErrorAlert from '../../components/ErrorAlert.svelte';

    const mostDownloadedPromise = request<MostDownloaded>('/web/v1/crates/most-downloaded');
    const recentlyCreatedPromise = request<RecentlyCreated>('/web/v1/crates/recently-created');
    const recentlyUpdatedPromise = request<RecentlyUpdated>('/web/v1/crates/recently-updated');
</script>

<header>
    <div class="container flex items-center mx-auto flex-col md:flex-row">
        <div class="p-10 pt-0 md:pt-10">
            <h1 class="text-5xl font-bold tracking-tight">
                Welcome to <span class="text-highlight">Chartered</span>.
            </h1>
            <h2>
                A private, authenticated Cargo registry. Everything published to this registry is private and visible
                only to you, until explicit permissions are granted to others.
            </h2>

            <a href="https://book.chart.rs/" target="_blank" class="block btn-blue-outline"> Learn More </a>
        </div>

        <div class="pr-6 order-first md:order-last pt-10 md:pt-0">
            <img alt="Rust logo" class="w-[8rem]" src={rustLogo} />️️
        </div>
    </div>
</header>

<main class="container mx-auto p-10 pt-0">
    <div class="block md:grid grid-cols-3 gap-9">
        <div>
            <h3 class="text-3xl mb-2">Newly created</h3>

            {#await recentlyCreatedPromise}
                {#each [1, 2, 3, 4, 5] as _}
                    <FeaturedCrate crate={null}>
                        <div class="my-0.5 flex items-center space-x-1">
                            <Icon name="calendar" />
                            <div class="skeleton w-16" />
                        </div>
                    </FeaturedCrate>
                {/each}
            {:then recentlyCreated}
                {#each recentlyCreated.crates as crate}
                    <FeaturedCrate {crate}>
                        <Icon name="calendar" />
                        <RelativeTime time={crate.created_at} />
                    </FeaturedCrate>
                {/each}
            {:catch e}
                <ErrorAlert showClose={false}>{e}</ErrorAlert>
            {/await}
        </div>

        <div>
            <h3 class="text-3xl mb-2">Recently updated</h3>

            {#await recentlyUpdatedPromise}
                {#each [1, 2, 3, 4, 5] as _}
                    <FeaturedCrate crate={null}>
                        <div class="skeleton w-9 my-1" />
                    </FeaturedCrate>
                {/each}
            {:then recentlyUpdated}
                {#each recentlyUpdated.versions as crate}
                    <FeaturedCrate {crate}>
                        v{crate.version}
                    </FeaturedCrate>
                {/each}
            {:catch e}
                <ErrorAlert showClose={false}>{e}</ErrorAlert>
            {/await}
        </div>

        <div>
            <h3 class="text-3xl mb-2">Most downloaded</h3>

            {#await mostDownloadedPromise}
                {#each [1, 2, 3, 4, 5] as _}
                    <FeaturedCrate crate={null}>
                        <div class="my-0.5 flex items-center space-x-1">
                            <Icon name="download" />
                            <div class="skeleton w-8" />
                        </div>
                    </FeaturedCrate>
                {/each}
            {:then mostDownloaded}
                {#each mostDownloaded.crates as crate}
                    <FeaturedCrate {crate}>
                        <Icon name="download" />
                        <span>{crate.downloads}</span>
                    </FeaturedCrate>
                {/each}
            {:catch e}
                <ErrorAlert showClose={false}>{e}</ErrorAlert>
            {/await}
        </div>
    </div>
</main>
