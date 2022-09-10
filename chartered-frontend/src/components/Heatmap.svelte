<script type="typescript">
    // @ts-expect-error missing typedefs
    import SvelteHeatmap from 'svelte-heatmap';
    import { DateTime } from 'luxon';
    import { onDestroy } from 'svelte';

    export let data: Promise<{ [date: string]: number }>;

    let heatmapData: { date: Date; value: number }[] = [];
    $: data.then((v) => {
        heatmapData = Object.entries(v).map(([k, v]) => ({ date: new Date(k), value: v }));
    });

    // we want to apply conditional styles to SvelteHeatmap which doesn't allow for css classes to be
    // applied
    let prefersDarkMode = window.matchMedia('(prefers-color-scheme: dark)');
    let isDarkMode = prefersDarkMode.matches;
    const onDarkModeChange = (v: MediaQueryListEvent) => (isDarkMode = v.matches);
    prefersDarkMode.addEventListener('change', onDarkModeChange);
    onDestroy(() => prefersDarkMode.removeEventListener('change', onDarkModeChange));
</script>

<div class="grid">
    <div class="overlay heatmap">
        <SvelteHeatmap
            allowOverflow={false}
            cellGap={5}
            fontFamily="inherit"
            fontColor={isDarkMode ? '#94a3b8' : 'black'}
            cellRadius={'50%'}
            data={[]}
            emptyColor={isDarkMode ? '#475569' : '#e2e8f0'}
            startDate={DateTime.now().minus({ year: 1 }).toJSDate()}
            endDate={DateTime.now().toJSDate()}
        />
    </div>

    <!-- hack to fade in colours over the empty heatmap -->
    <div
        class:!opacity-100={heatmapData.length !== 0}
        class="overlay heatmap opacity-0 transition-all ease-out duration-100"
    >
        <SvelteHeatmap
            allowOverflow={false}
            cellGap={5}
            fontFamily="inherit"
            fontColor={isDarkMode ? '#94a3b8' : 'black'}
            cellRadius={'50%'}
            data={heatmapData}
            emptyColor={isDarkMode ? '#475569' : '#e2e8f0'}
            startDate={DateTime.now().minus({ year: 1 }).toJSDate()}
            endDate={DateTime.now().toJSDate()}
        />
    </div>
</div>

<style lang="postcss">
    .overlay {
        grid-area: 1 / 1;
    }
</style>
