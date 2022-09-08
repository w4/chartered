<script type="typescript">
    import type { Crate } from '../types/featured_crate';
    import Icon from './Icon.svelte';

    /**
     * The crate to show in the card
     */
    export let crate: Crate;
</script>

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
