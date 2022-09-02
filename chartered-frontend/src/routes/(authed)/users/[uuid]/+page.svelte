<script type="typescript">
    import { page } from '$app/stores';
    import { request } from '../../../../stores/auth';
    import Spinner from '../../../../components/Spinner.svelte';
    import Icon from '../../../../components/Icon.svelte';
    import type { User } from '../../../../types/user';

    let userPromise;
    $: userPromise = request<User>(`/web/v1/users/info/${$page.params.uuid}`).then(
        (user: User & { displayName: string }) => {
            user.displayName = user.nick || user.name || user.username;
            return user;
        },
    );
</script>

{#await userPromise}
    <header>
        <div class="relative h-[20rem]">
            <Spinner />
        </div>
    </header>
{:then user}
    <header>
        <div class="container flex flex-col md:flex-row items-center md:items-start mx-auto p-10 mb-3">
            <div class="flex-grow text-center md:text-left">
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
            </div>

            <div class="order-first md:order-last">
                {#if user.picture_url}
                    <img alt="Your profile picture" src={user.picture_url} class="rounded-[50%] h-[8rem] inline" />
                {:else}
                    <div
                        class="h-[8rem] w-[8rem] rounded-[50%] text-gray-300 bg-gray-100 dark:bg-gray-800 overflow-hidden"
                    >
                        <Icon height="8rem" width="8rem" name="user" />
                    </div>
                {/if}
            </div>
        </div>
    </header>
{/await}
