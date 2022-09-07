<script type="typescript">
    import type { VersionDependency } from '../../../../../types/crate';

    /**
     * The dependency to show
     */
    export let dependency: VersionDependency;

    /**
     * CSS classes to apply to the outer div drawn by this component.
     */
    let clazz = '';
    export { clazz as class };

    /**
     * Extracts the "organisation" part of an SSH url if one exists, otherwise returns the
     * full URL. This currently assumes that all dependencies with an SSH URI are dependencies
     * local to this instance of chartered, but that's not always the case.
     */
    function getLocalDependencyOrganisation(): string | undefined {
        const s = dependency.registry?.split('/');

        return s ? s[s.length - 1] : dependency.registry;
    }
</script>

<div class={clazz}>
    {#if dependency.registry === 'https://github.com/rust-lang/crates.io-index'}
        <a href={`https://crates.io/crates/${dependency.name}`} target="_blank">
            {dependency.name}
        </a>
    {:else if dependency.registry?.indexOf('ssh://') === 0}
        <a href={`/crates/${getLocalDependencyOrganisation()}/${dependency.name}`} target="_blank">
            {dependency.name}
        </a>
    {:else}
        {dependency.name}
    {/if}
    = "{dependency.req}"
</div>
