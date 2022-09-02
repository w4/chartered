<script type="typescript">
    import { request } from '../../../../../stores/auth';
    import { page } from '$app/stores';
    import AddMember from '../AddMember.svelte';
    import Member from '../Member.svelte';

    let newMember = null;

    let membersPromise;
    $: membersPromise = request(`/web/v1/crates/${$page.params.organisation}/${$page.params.crate}/members`);

    function reloadMembers() {
        newMember = null;
        membersPromise = request(`/web/v1/crates/${$page.params.organisation}/${$page.params.crate}/members`);
    }
</script>

{#await membersPromise then members}
    <div class="card-divide divide-y">
        {#each members.members as member, i}
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
