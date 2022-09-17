<script type="typescript">
    import type { Crate } from '../../../../../types/crate';
    import { page } from '$app/stores';

    let version = '*';
    const updateVersion = (vers: string) => (version = vers || '*');

    export let cratePromise: Promise<Crate>;
    $: {
        updateVersion('*');
        cratePromise.then((crate) => updateVersion(crate?.versions[0]?.vers));
    }
</script>

[dependencies]
{$page.params.crate} = {`{ version = "${version}", registry = "${$page.params.organisation}" }`}
