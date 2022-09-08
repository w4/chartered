<script type="typescript">
    import { page } from '$app/stores';
    import '../app.pcss';

    const defaultPageTitle = 'Chartered';

    let pageTitle: string;
    $: if ($page.data.title) {
        pageTitle = `${$page.data.title} - ${defaultPageTitle}`;
    } else {
        pageTitle = defaultPageTitle;
    }
</script>

<svelte:head>
    <title>{pageTitle || defaultPageTitle}</title>
</svelte:head>

<slot />
