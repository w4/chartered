<script type="typescript">
    import feather from 'feather-icons';
    export const directions = ['n', 'ne', 'e', 'se', 's', 'sw', 'w', 'nw'];

    /**
     * Name of the icon, can be found on https://feathericons.com/
     */
    export let name: string;

    /**
     * Compass rotation of the icon
     */
    export let direction = 'n';

    /**
     * Thickness of the icon
     */
    export let strokeWidth: string | null = null;

    /**
     * Colour of the icon
     */
    export let stroke: string | null = null;

    /**
     * Width of the icon
     */
    export let width = '1em';

    /**
     * Height of the icon
     */
    export let height = '1em';

    $: icon = feather.icons[name];
    $: rotation = directions.indexOf(direction) * 45;
    $: if (icon) {
        if (stroke) icon.attrs['stroke'] = stroke;
        if (strokeWidth) icon.attrs['stroke-width'] = strokeWidth;
    }
</script>

{#if icon}
    <svg {...icon.attrs} style="width: {width}; height: {height}; transform: rotate({rotation}deg);">
        <g>
            {@html icon.contents}
        </g>
    </svg>
{/if}

<style>
    svg {
        width: 1em;
        height: 1em;
        overflow: visible;
        transform-origin: 50% 50%;
    }
</style>
