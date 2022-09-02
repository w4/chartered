<script type="typescript">
    import { auth, logout } from '../../stores/auth';
    import { goto } from '$app/navigation';
    import Nav from '../../components/Nav.svelte';
    import NavItem from '../../components/NavItem.svelte';

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
