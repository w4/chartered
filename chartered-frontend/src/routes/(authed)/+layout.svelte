<script type="typescript">
    import { auth } from '../../stores/auth';
    import { goto } from '$app/navigation';
    import Nav from '../../components/Nav.svelte';
    import NavItem from '../../components/NavItem.svelte';

    // watch the `$auth` store for changes to authentication, if their `$auth` disappears
    // (such as from expiry), redirect to the login page. this also covers the case where
    // the user requests `/`, we'll redirect straight to login from this too.
    $: if (!$auth) {
        goto('/auth/login', { replaceState: true });
    }
</script>

{#if $auth}
    <Nav>
        <NavItem href="/">Home</NavItem>
        <NavItem href="/ssh-keys">SSH Keys</NavItem>
        <NavItem href="/organisations" aliases={['/crates']}>Organisations</NavItem>
    </Nav>

    <slot />
{/if}
