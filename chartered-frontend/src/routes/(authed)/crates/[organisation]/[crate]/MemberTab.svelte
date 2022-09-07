<script type="typescript">
    import { request } from '../../../../../stores/auth';
    import { page } from '$app/stores';
    import AddMember from '../AddMember.svelte';
    import Member from '../Member.svelte';
    import type { CrateMembers, CrateMember } from '../../../../../types/crate';

    /**
     * Binding to the latest user selected after a search, this is a "prospective member" that is not yet
     * persisted to the backend until the user presses the save button.
     */
    let newMember: CrateMember | null = null;

    // Grab all the current crate's members
    let membersPromise: Promise<CrateMembers>;
    $: membersPromise = request(`/web/v1/crates/${$page.params.organisation}/${$page.params.crate}/members`);

    /**
     * When a member is updated/added/deleted to this crate, we'll want to reload to show the user exactly
     * what the server sees the current state is.
     */
    function reloadMembers() {
        newMember = null;
        membersPromise = request(`/web/v1/crates/${$page.params.organisation}/${$page.params.crate}/members`);
    }
</script>

{#await membersPromise then members}
    <div class="card-divide divide-y">
        {#each members.members as member}
            <Member
                {member}
                organisation={$page.params.organisation}
                crate={$page.params.crate}
                possiblePermissions={members.possible_permissions}
                on:updated={reloadMembers}
            />
        {/each}

        {#if newMember}
            <Member
                member={newMember}
                newPermissions={['VISIBLE']}
                organisation={$page.params.organisation}
                crate={$page.params.crate}
                possiblePermissions={members.possible_permissions}
                on:updated={reloadMembers}
            />
        {/if}

        <div class="p-6">
            <AddMember
                hideUuids={members.members.map((v) => v.uuid)}
                on:new={(member) => {
                    member.detail.permissions = [];
                    member.detail.uuid = member.detail.user_uuid;
                    newMember = member.detail;
                }}
            />
        </div>
    </div>
{/await}
