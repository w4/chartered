<script type="typescript">
    import type { Version } from '../../../../../types/crate';
    import Icon from '../../../../../components/Icon.svelte';
    import RelativeTime from '../../../../../components/RelativeTime.svelte';

    let clazz = '';

    export let version: Version;
    export { clazz as class };

    function humanFileSize(size: number): string {
        const i = Math.floor(Math.log(size) / Math.log(1024));
        return Number((size / Math.pow(1024, i)).toFixed(2)) + ' ' + ['B', 'kB', 'MB', 'GB', 'TB'][i];
    }
</script>

<div class={clazz}>
    <div class="card-header">{version.vers}</div>

    <div class="card-body space-x-2">
        <div class="inline-flex items-center space-x-1">
            <span>By</span>

            {#if version.uploader.picture_url}
                <img
                    alt={version.uploader.display_name}
                    src={version.uploader.picture_url}
                    class="rounded-[50%] h-[1rem] w-[1rem]"
                />
            {:else}
                <div class="h-[1rem] w-[1rem] rounded-[50%] text-gray-300 bg-gray-100 dark:bg-gray-800 overflow-hidden">
                    <Icon height="1rem" width="1rem" name="user" />
                </div>
            {/if}

            <a
                href={`/users/${version.uploader.uuid}`}
                class="border-b border-blue-500 text-blue-500 hover:border-blue-600 hover:text-blue-600"
            >
                {version.uploader.display_name}
            </a>
        </div>

        <div class="inline-flex items-center space-x-1">
            <Icon name="calendar" />
            <RelativeTime time={version.created_at} />
        </div>

        <div class="inline-flex items-center space-x-1">
            <Icon name="hard-drive" />
            <span>{humanFileSize(version.size)}</span>
        </div>

        <div class="inline-flex items-center space-x-1">
            <Icon name="check-square" />
            <span>{Object.keys(version.features).length} features</span>
        </div>
    </div>
</div>
