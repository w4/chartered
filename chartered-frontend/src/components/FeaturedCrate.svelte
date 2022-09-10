<script type="typescript">
    import type { Crate } from '../types/featured_crate';
    import Icon from './Icon.svelte';

    /**
     * The crate to show in the card
     */
    export let crate: Crate | null;
</script>

{#if crate}
    <a href={`/crates/${crate.organisation}/${crate.name}`} class="card flex items-center mb-3">
        <div class="flex-grow">
            <div class="font-semibold">
                {crate.organisation}/<span class="text-highlight">{crate.name}</span>
            </div>

            <div class="text-xs flex items-center space-x-1">
                <slot />
            </div>
        </div>

        <Icon name="chevron-right" strokeWidth="3px" />
    </a>
{:else}
    <div class="card flex items-center mb-3">
        <div class="flex-grow">
            <div class="flex items-end pb-2 h-6">
                <div style={`width: ${Math.random() * 4 + 4}rem`} class="skeleton" />
                <div style={`width: ${Math.random() * 4 + 4}rem`} class="ml-2 skeleton-highlight" />
            </div>

            <div class="text-xs flex items-center space-x-1">
                <slot />
            </div>
        </div>
    </div>
{/if}
