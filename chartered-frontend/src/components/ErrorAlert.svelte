<script type="typescript">
    import { createEventDispatcher } from 'svelte';
    import Icon from './Icon.svelte';

    /**
     * Sets whether the close button should be shown on this error alert.
     */
    export let showClose = true;

    // create the event dispatcher, so we can tell the caller when the user attempts
    // to close the prompt, so it can clear the message and hide us from display
    const dispatch = createEventDispatcher();
</script>

<div class="flex p-4 mb-2 text-sm text-red-700 bg-red-100 rounded-lg dark:bg-red-200 dark:text-red-800" role="alert">
    <div class="flex-grow flex items-center">
        <slot />
    </div>

    <div class:hidden={!showClose}>
        <button
            type="button"
            on:click={() => dispatch('close')}
            class="bg-red-100 text-red-500 rounded-lg focus:ring-2 focus:ring-red-400 p-1.5 hover:bg-red-200 inline-flex dark:bg-red-200 dark:text-red-600 dark:hover:bg-red-300"
            aria-label="Close"
        >
            <span class="sr-only">Close</span>
            <Icon name="x" />
        </button>
    </div>
</div>
