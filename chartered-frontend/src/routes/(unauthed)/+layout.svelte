<script type="typescript">
    import { auth } from '../../stores/auth.ts';
    import { goto } from '$app/navigation';

    $: if ($auth) {
        goto('/', { replaceState: true });
    }
</script>

{#if !$auth}
    <div class="h-screen flex flex-col justify-center items-center">
        <div class="w-full md:w-[40rem] p-2 md:p-0">
            <header class="bg-none">
                <h1 class="text-2xl font-bold">chartered ✈️</h1>
                <h6 class="text-sm">a private, authenticated cargo registry</h6>
            </header>

            <main>
                <slot />
            </main>
        </div>
    </div>
{/if}

<style>
    main {
        @apply w-full mt-2 p-4 bg-white dark:bg-transparent border dark:border-slate-700 rounded relative;
    }
</style>
