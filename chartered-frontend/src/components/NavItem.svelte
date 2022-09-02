<script type="typescript">
    import { page } from '$app/stores';

    /**
     * URL to navigate to after pressing the link
     */
    export let href: string;

    /**
     * A list of base URLs that should cause this navigation item to be marked as
     * the active page.
     */
    export let aliases: string[] = [];

    let active: boolean;
    $: if (href === '/') {
        active = $page.url.pathname === href;
    } else {
        active = $page.url.pathname.startsWith(href) || aliases.some((v) => $page.url.pathname.startsWith(v));
    }
</script>

<li>
    {#if active}
        <a
            {href}
            class="block py-2 pr-4 pl-3 text-white bg-blue-700 rounded md:bg-transparent md:text-blue-700 md:p-0 dark:text-white"
            aria-current="page"
        >
            <slot />
        </a>
    {:else}
        <a
            {href}
            class="block py-2 pr-4 pl-3 text-gray-700 rounded hover:bg-gray-100 md:hover:bg-transparent md:hover:text-blue-700 md:p-0 md:dark:hover:text-white dark:text-gray-400 dark:hover:bg-gray-700 dark:hover:text-white md:dark:hover:bg-transparent dark:border-gray-700"
        >
            <slot />
        </a>
    {/if}
</li>
