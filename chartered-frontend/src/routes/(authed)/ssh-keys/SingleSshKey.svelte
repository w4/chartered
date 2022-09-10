<script type="typescript">
    import type { SshKey } from '../../../types/ssh_keys';
    import RelativeTime from '../../../components/RelativeTime.svelte';
    import Icon from '../../../components/Icon.svelte';
    import { createEventDispatcher } from 'svelte';

    /**
     * The SSH key to draw up some information about
     */
    export let sshKey: SshKey | null;

    // build the event dispatcher for alerting the main view that the user wants to
    // delete the current key
    const dispatch = createEventDispatcher();
</script>

<div class="p-2 flex items-center">
    <div class="flex-grow">
        <h3 class="text-lg">
            {#if sshKey}
                {sshKey.name}
            {:else}
                <div class="skeleton w-24 mt-3" />
            {/if}
        </h3>

        {#if sshKey}
            <code class="text-xs">
                {sshKey.fingerprint}
            </code>
        {:else}
            <div class="flex">
                {#each [...Array(32).keys()] as _}
                    <div class="skeleton w-4 mt-4 mr-1.5" />
                {/each}
            </div>
        {/if}

        <div class="text-xs pt-0.5 pb-1">
            {#if sshKey}
                <span>Added <RelativeTime time={sshKey.created_at} /></span>

                <span class="ml-2">
                    Last used
                    {#if sshKey.last_used_at}
                        <RelativeTime time={sshKey.last_used_at} />
                    {:else}
                        never
                    {/if}
                </span>
            {:else}
                <div class="flex mt-3 mb-1">
                    <div class="skeleton w-32" />
                    <div class="skeleton w-24 ml-2" />
                </div>
            {/if}
        </div>
    </div>

    <div class="pr-3">
        <button class="text-red-700" on:click={() => dispatch('delete')}>
            <Icon name="trash" />
        </button>
    </div>
</div>
