<script type="typescript">
    import Icon from '../../../../components/Icon.svelte';
    import ErrorAlert from '../../../../components/ErrorAlert.svelte';
    import { BASE_URL, auth } from '../../../../stores/auth';
    import { goto } from '$app/navigation';
    import Spinner from '../../../../components/Spinner.svelte';

    /**
     * Binding to the name field in the form.
     */
    let name = '';

    /**
     * Binding to the description field in the form.
     */
    let description = '';

    /**
     * Binding to the public checkbox in the form.
     */
    let isPublic = false;

    /*
     * Disables the form and replaces the submit button with a spinner if true.
     */
    let submitting = false;

    /**
     * The last error returned from the backend as a result of the submission. If this is
     * not null then the user is displayed an alert.
     */
    let error: string | null = null;

    /**
     * Submit the form data to the backend to attempt to create the organisation.
     */
    async function submit() {
        // clear the error since it'll no longer be applicable after this submission
        error = '';

        // disable the form and show a spinner to the user
        submitting = true;

        try {
            // attempt the actual creation of the organisation
            let result = await fetch(`${BASE_URL}/a/${$auth.auth_key}/web/v1/organisations`, {
                method: 'PUT',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({ name, description, public: isPublic }),
            });
            let json = await result.json();

            if (json.error) {
                throw new Error(json.error);
            }

            // successful creation, return the user back to the list
            await goto('/organisations/list');
        } catch (e) {
            error = e.toString();
        } finally {
            // stop showing the spinner once creation is complete
            submitting = false;
        }
    }
</script>

<header>
    <div class="container flex items-center mx-auto p-10">
        <h1 class="text-5xl font-bold tracking-tight">
            Create a new <span class="text-highlight">Organisation</span>.
        </h1>
    </div>
</header>

<main class="container mx-auto p-10 pt-0">
    {#if error}
        <ErrorAlert on:close={() => (error = null)}>{error}</ErrorAlert>
    {/if}

    <form on:submit|preventDefault={submit} class="card mb-4 p-4">
        <div class="mb-4">
            <label for="name" class="block mb-1 text-sm font-medium">Name</label>
            <input
                bind:value={name}
                required
                type="text"
                id="name"
                placeholder="my-backend-team"
                pattern="[a-zA-Z0-9-]*"
                class="border border-gray-200 dark:border-gray-700 bg-transparent text-sm rounded-lg block w-full p-2.5 ring-blue-500 focus:border-blue-500"
            />
            <div class="text-xs mt-1">Must be in the format <code>[a-zA-Z0-9-]*</code></div>
        </div>

        <div class="mb-4">
            <label for="description" class="block mb-1 text-sm font-medium">Description</label>
            <textarea
                bind:value={description}
                rows={3}
                id="description"
                class="border border-gray-200 dark:border-gray-700 bg-transparent text-sm rounded-lg block w-full p-2.5 ring-blue-500 focus:border-blue-500"
            />
        </div>

        <div class="mb-4">
            <input
                bind:checked={isPublic}
                type="checkbox"
                id="public"
                class="w-4 h-4 rounded border border-gray-200 dark:border-gray-700 bg-transparent ring-blue-500 focus:border-blue-500 !ring-offset-0"
            />
            <label for="public" class="text-sm">
                Grant all users access to view and download crates from this organisation (cannot be removed after
                creation)
            </label>
        </div>

        {#if submitting}
            <div class="relative h-4 w-4">
                <Spinner />
            </div>
        {:else}
            <button
                type="submit"
                class="inline-flex items-center py-2.5 px-4 text-xs font-medium text-center text-white bg-blue-700 rounded-lg focus:ring-4 focus:ring-blue-200 dark:focus:ring-blue-900 hover:bg-blue-800"
            >
                <Icon name="plus" />
                <span class="ml-1">Create</span>
            </button>
        {/if}
    </form>
</main>
