<script type="typescript">
    import { page } from '$app/stores';
    import { handleOAuthCallback } from '../../../../stores/auth';
    import Spinner from '../../../../components/Spinner.svelte';
    import ErrorAlert from '../../../../components/ErrorAlert.svelte';

    // pass the payload onto the backend to verify and create a session, we'll just show a
    // spinner in the meantime.
    const callback = handleOAuthCallback($page.url.search);
</script>

<div class="h-[18rem]">
    {#await callback}
        <Spinner />
    {:then}
        <Spinner />
    {:catch error}
        <!-- todo: redirect back to login -->
        <ErrorAlert showClose={false}>{error}</ErrorAlert>
    {/await}
</div>
