<script>
    import { onMount } from 'svelte';
    import { page } from '$app/stores';
    import { goto } from '$app/navigation';

    let feedback = [];
    let includeRead = false;
    let isLoading = true;
    let isAdmin = false;

    $: {
        const adminParam = $page.url.searchParams.get('admin');
        isAdmin = adminParam?.toUpperCase() === 'TRUE';
        if (!isAdmin) goto('/'); // redirect non-admins
    }

    async function loadFeedback() {
        isLoading = true;
        const res = await fetch(`/api/feedback${includeRead ? '' : '?include_read=false'}`);
        feedback = await res.json();
        feedback.sort((a, b) => b.timestamp - a.timestamp);
        isLoading = false;
    }

    async function markRead(id) {
        await fetch(`/api/feedback/${id}`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ read: true })
        });
        await loadFeedback();
    }

    async function deleteFeedback(id) {
        await fetch(`/api/feedback/${id}`, { method: 'DELETE' });
        await loadFeedback();
    }

    async function assignFeedback(id, assignee) {
        await fetch(`/api/feedback/${id}`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ assigned_to: assignee })
        });
        await loadFeedback();
    }

    onMount(loadFeedback);
</script>

<svelte:head>
    <title>Admin Feedback</title>
</svelte:head>

<div class="max-w-4xl mx-auto p-4">
    <!-- Header -->
    <header class="mb-6">
        <h1 class="text-2xl font-bold text-unifest-green-dark mb-2">
            Admin Feedback Panel
        </h1>

        <div class="bg-red-100 border border-red-400 text-red-700 px-4 py-2 rounded mb-4">
            <h2 class="font-bold">ADMIN MODE</h2>
        </div>

        <div class="flex justify-between items-center mb-4">
            <a
                href="/?admin=true"
                class="inline-block text-sm px-4 py-2 rounded-md font-semibold transition text-unifest-green-dark border border-unifest-green-dark hover:bg-unifest-green hover:text-white"
            >
                Meals
            </a>
        </div>

        <div class="flex items-center my-2">
            <label class="flex items-center">
                <input type="checkbox" bind:checked={includeRead} on:change={loadFeedback} class="mr-2 hover:shadow">
                Show read feedback
            </label>
        </div>
    </header>

    <!-- Loading State -->
    {#if isLoading}
        <div class="bg-unifest-green bg-opacity-20 p-4 rounded-md animate-pulse">
            <p>Loading feedback...</p>
            <p class="text-sm text-gray-600">
                (If this takes a while, something might be wrong 😕)
            </p>
        </div>
    {:else if feedback.length === 0}
        <div class="bg-yellow-100 border border-yellow-400 text-yellow-700 px-4 py-3 rounded mb-4">
            {#if includeRead}
                <p>No feedback available.</p>
            {:else}
                <p>No unread feedback available.</p>
            {/if}
        </div>
    {:else}
        <div class="text-2xl font-bold text-unifest-green-dark mb-4">
            {#if includeRead}
                <h1>All Feedback:</h1>
            {:else}
                <h1>Unread Feedback:</h1>
            {/if}
        </div>
        <!-- Feedback Entries -->
        {#each feedback as f}
            <div class="mb-6">
                <h2 class="text-xl font-semibold bg-unifest-green-dark text-white px-4 py-2 rounded-t-md">
                    {new Date(f.timestamp * 1000).toLocaleString('de-DE', {
                        weekday: 'long',
                        day: '2-digit',
                        month: '2-digit',
                        year: 'numeric',
                        hour: '2-digit',
                        minute: '2-digit',
                        })}
                </h2>
                <div class="flex flex-col gap-2 p-4 bg-unifest-green bg-opacity-10 rounded-b-md">
                    <p><strong>Feedback:</strong> {f.feedback}</p>
                    {#if f.assigned_to != null}
                        <p><strong>Assigned To:</strong> {f.assigned_to || '—'}</p>
                    {/if}
                    <div class="flex flex-wrap gap-3 mt-2">
                        <button
                            on:click={() => markRead(f.feedback_id)}
                            class="bg-blue-600 hover:bg-blue-700 text-white px-3 py-1 rounded hover:shadow"
                        >Mark as Read</button>

                        <button
                            on:click={() => deleteFeedback(f.feedback_id)}
                            class="bg-red-600 hover:bg-red-700 text-white px-3 py-1 rounded hover:shadow"
                        >Delete</button>

                        <input
                            type="text"
                            placeholder="Assign to..."
                            on:change={(e) => assignFeedback(f.feedback_id, e.target.value)}
                            class="border border-gray-300 rounded px-2 py-1 hover:shadow"
                        />
                    </div>
                </div>
            </div>
        {/each}
    {/if}
</div>
