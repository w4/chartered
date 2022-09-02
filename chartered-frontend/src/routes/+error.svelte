<script type="typescript">
    import { page } from '$app/stores';
</script>

<div class="flex flex-col items-center justify-center text-center h-screen">
    <h1 class="text-5xl font-bold tracking-light">{$page.status}</h1>
    <h2 class="text-xl tracking-light">{$page.error.message}</h2>
</div>
