<script type="typescript">
    import { page } from '$app/stores';

    let indexUri = '';
    $: indexUri = `${import.meta.env.VITE_CHARTERED_SSH_URL}${$page.params.organisation}`;
</script>

[registries]
{$page.params.organisation} = {`{ index = "${indexUri}" }`}
