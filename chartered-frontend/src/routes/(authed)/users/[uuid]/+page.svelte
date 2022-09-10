<script type="typescript">
    import { page } from '$app/stores';
    import { request } from '../../../../stores/auth';
    import Spinner from '../../../../components/Spinner.svelte';
    import Icon from '../../../../components/Icon.svelte';
    import type { User } from '../../../../types/user';
    import Heatmap from '../../../../components/Heatmap.svelte';

    // this is used for conditionally aligning the spinner to the center of the header for aesthetics
    let isLoaded = false;

    // grabs the requested user from the backend and determine a `displayName` for them to show on their
    // profile.
    let userPromise: Promise<User & { displayName?: string }>;
    $: {
        isLoaded = false;
        userPromise = request<User>(`/web/v1/users/info/${$page.params.uuid}`).then(
            (user: User & { displayName?: string }) => {
                isLoaded = true;
                user.displayName = user.nick || user.name || user.username;
                return user;
            },
        );
    }

    // grab data for the user's heatmap from the API
    let heatmapPromise: Promise<{ [date: string]: number }>;
    $: heatmapPromise = request(`/web/v1/users/info/${$page.params.uuid}/heatmap`);
</script>

<header>
    <div class:md:items-start={isLoaded} class="container flex flex-col md:flex-row items-center mx-auto p-10 mb-3">
        <div class="flex-grow text-center md:text-left">
            {#await userPromise}
                <div class="relative justify-center w-12 h-12">
                    <Spinner />
                </div>
            {:then user}
                <h1 class="text-5xl text-highlight font-bold tracking-tight">
                    {user.displayName}
                </h1>

                <h2 class="space-x-2">
                    {#each [user.nick, user.name, user.username] as alias}
                        {#if alias && user.displayName !== alias}
                            <span>aka <strong>{alias}</strong></span>
                        {/if}
                    {/each}
                </h2>

                {#if user.external_profile_url}
                    <a
                        href={user.external_profile_url}
                        target="_blank"
                        class="inline-flex space-x-1 items-center btn-blue-outline"
                    >
                        <Icon name="home" />
                        <span>Homepage</span>
                    </a>
                {/if}

                {#if user.email}
                    <a href={`mailto:${user.email}`} class="inline-flex space-x-1 items-center btn-blue-outline">
                        <Icon name="mail" />
                        <span>Email</span>
                    </a>
                {/if}
            {/await}
        </div>

        <div class="order-first md:order-last">
            {#await userPromise}
                <div class="h-[8rem] w-[8rem] rounded-[50%] text-gray-300 bg-gray-100 dark:bg-gray-800 overflow-hidden">
                    <Icon height="8rem" width="8rem" name="user" />
                </div>
            {:then user}
                {#if user.picture_url}
                    <img alt={user.displayName} src={user.picture_url} class="rounded-[50%] h-[8rem] inline" />
                {:else}
                    <div
                        class="h-[8rem] w-[8rem] rounded-[50%] text-gray-300 bg-gray-100 dark:bg-gray-800 overflow-hidden"
                    >
                        <Icon height="8rem" width="8rem" name="user" />
                    </div>
                {/if}
            {/await}
        </div>
    </div>
</header>

<main class="container h-fit mx-auto p-10 pt-0">
    <div class="card">
        <Heatmap data={heatmapPromise} />
    </div>
</main>
