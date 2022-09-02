<script type="typescript">
    import type { VersionDependency } from '../../../../../types/crate';

    let clazz = '';

    export let dependency: VersionDependency;
    export { clazz as class };

    function getLocalDependencyOrganisation(registry: string): string {
        const s = dependency.registry.split('/');

        return s[s.length - 1];
    }
</script>

<div class={clazz}>
    {#if dependency.registry === 'https://github.com/rust-lang/crates.io-index'}
        <a href={`https://crates.io/crates/${dependency.name}`} target="_blank">
            {dependency.name}
        </a>
    {:else if dependency.registry.indexOf('ssh://') === 0}
        <a href={`/crates/${getLocalDependencyOrganisation(dependency.registry)}/${dependency.name}`} target="_blank">
            {dependency.name}
        </a>
    {:else}
        {dependency.name}
    {/if}
    = "{dependency.req}"
</div>
