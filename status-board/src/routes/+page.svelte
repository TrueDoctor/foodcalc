<script>
    import { page } from '$app/stores';
    import Meal from '$lib/Meal.svelte';
    import { onMount } from 'svelte';
    import { goto } from '$app/navigation';
    import { 
        fetchMeals, 
        fetchEvents, 
        fetchEventDetails, 
        isMealActive, 
        formatDate, 
        getDayKey 
    } from '$lib/api.ts';

    // Get event ID from URL or null if not specified
    let eventId = $page.url.searchParams.get('event');
    if (eventId) eventId = parseInt(eventId, 10);
    
    // Check for noselect parameter, but don't apply it in admin mode
    const noSelectParam = $page.url.searchParams.get('noselect') !== null;
    const adminPassword = $page.url.searchParams.get('admin');
    const isAdmin = adminPassword?.toUpperCase() === "TRUE";
    // Only apply noSelect if we're not in admin mode
    const noSelect = noSelectParam && !isAdmin;
    
    let days = [];
    let events = [];
    let activeEvents = [];
    let isLoading = true;
    let error = null;
    let currentEventName = '';
    let isPastEvent = false;

    async function loadEvents() {
        try {
            events = await fetchEvents();
            
            // In admin mode, all events are considered active
            if (isAdmin) {
                activeEvents = [...events]; // Create a copy of all events
                return;
            }
            
            // Fetch meals for each event to determine which ones have active meals
            for (const event of events) {
                try {
                    const eventMeals = await fetchMeals(event.id);
                    
                    // Check if any meals in this event are active (not in the past)
                    const hasActiveMeals = eventMeals.some(day => 
                        day.some(meal => isMealActive(meal))
                    );
                    
                    if (hasActiveMeals) {
                        activeEvents.push(event);
                    }
                } catch (err) {
                    console.error(`Could not fetch meals for event ${event.id}:`, err);
                }
            }
        } catch (err) {
            console.error('Error fetching events:', err);
        }
    }
    
    async function fetchData() {
        isLoading = true;
        error = null;
        isPastEvent = false;
        
        try {
            // Fetch meals
            const mealsData = await fetchMeals(eventId);
            
            // Check if we have any meals
            if (mealsData.length === 0 || mealsData.every(day => day.length === 0)) {
                error = eventId 
                    ? `No meals found for event ${eventId}`
                    : 'No upcoming events found';
                days = [];
                return;
            }
            
            // Check if all meals are in the past
            const allMealsInPast = mealsData.every(day => 
                day.every(meal => !isMealActive(meal))
            );
            
            if (allMealsInPast) {
                isPastEvent = true;
            }
            
            // Use the grouping provided by the server directly
            days = mealsData;
            
            // Try to fetch event name if we have the event ID
            if (eventId || (days[0]?.[0]?.status?.event_id)) {
                const actualEventId = eventId || days[0][0].status.event_id;
                try {
                    const eventData = await fetchEventDetails(actualEventId);
                    currentEventName = eventData.name;
                } catch (err) {
                    console.error('Could not fetch event details:', err);
                    // Use a default name based on the first meal's date
                    if (days[0]?.[0]) {
                        const date = new Date(days[0][0].status.start * 1000);
                        currentEventName = `Event on ${formatDate(days[0][0].status.start, {
                            month: 'long',
                            day: 'numeric'
                        })}`;
                    }
                }
            }

            // Fetch unread feedback count
            await checkUnreadFeedback();
        } catch (err) {
            // Parse error message to be more user-friendly
            if (err.message.includes('404') || err.message.includes('No meals found')) {
                error = eventId 
                    ? `Event ${eventId} not found or has no meals`
                    : 'No upcoming events found';
            } else {
                error = `Error: ${err.message}`;
                console.error('Error fetching data:', err);
            }
            days = [];
        } finally {
            isLoading = false;
        }
    }
    
    function changeEvent(newEventId) {
        // Navigate to the selected event or home page if empty
        if (newEventId) {
            goto(`?event=${newEventId}`);
            eventId = parseInt(newEventId, 10);
        } else {
            goto(`/`);
            eventId = null;
        }
        fetchData();
    }
    
    onMount(() => {
        // Fetch available events
        loadEvents();
        
        // Initial data fetch
        fetchData();
        
        // Set up periodic refresh
        const interval = setInterval(fetchData, 10000); // Refresh every 10 seconds
        
        // Clean up interval on component unmount
        return () => clearInterval(interval);
    });
    // Filter meals to only show active ones by default
    let showPastMeals = false;
    
    // Define meal filter function
    $: meal_filter = (meal) => {
        if (showPastMeals) return true;
        return isMealActive(meal);
    };

    // New reactive state for feedback form
    let feedbackText = '';
    let feedbackStatus = '';  // 'success', 'error', or ''
    let feedbackMessage = '';

    async function submitFeedback() {
        feedbackStatus = '';
        feedbackMessage = '';

        if (feedbackText.trim().length < 10) {
            feedbackStatus = 'error';
            feedbackMessage = 'Please enter at least 10 characters of feedback.';
            return;
        }

        const payload = {
            feedback_id: 0, // This will be set by the server
            feedback: feedbackText.trim(),
            event_id: eventId ?? -1,
            timestamp: Math.floor(Date.now() / 1000),
            read: false,
            assigned_to: null // Initially no one is assigned
        };

        try {
            // Example: POST feedback to /api/feedback (adjust URL as needed)
            const res = await fetch('/api/feedback', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(payload),
            });

            if (!res.ok) {
                throw new Error(`Server responded with ${res.status}`);
            }

            feedbackStatus = 'success';
            feedbackMessage = 'Thank you for your feedback!';
            feedbackText = ''; // reset input
        } catch (err) {
            feedbackStatus = 'error';
            feedbackMessage = 'Failed to submit feedback. Please try again later.';
            console.error('Feedback submission error:', err);
        }
    }

    export async function fetchFeedback(includeRead = false) {
        const url = `/api/feedback${includeRead ? '?include_read=true' : ''}`;
        const res = await fetch(url);
        return await res.json();
    }

    
    let unreadFeedbackCount = 0;

    async function checkUnreadFeedback() {
        try {
            const res = await fetch('/api/feedback?include_read=false'); // or however your API is routed
            if (!res.ok) {
                unreadFeedbackCount = 0;
            }
            const response = await res.json();
            unreadFeedbackCount = response.length;
        } catch (err) {
            console.error('Failed to check unread feedback:', err);
        }
    }
</script>

<div class="max-w-4xl mx-auto p-4">
    <!-- Header -->
    <header class="mb-6">
        <h1 class="text-2xl font-bold text-unifest-green-dark mb-2">
            {#if currentEventName}
                {currentEventName} - Food Status
            {:else}
                Food Status Board
            {/if}
        </h1>
        
        {#if isAdmin}
            <div class="bg-red-100 border border-red-400 text-red-700 px-4 py-2 rounded mb-4">
                <h2 class="font-bold">ADMIN MODE</h2>
            </div>
                <div class="flex justify-between items-center mb-4">
        <a 
            href="/admin/feedback?admin=true" 
            class="inline-block text-sm px-4 py-2 rounded-md font-semibold transition 
                   hover:bg-unifest-green hover:text-white
                   {unreadFeedbackCount > 0 ? 'bg-blue-100 text-blue-800 border border-blue-800 hover:bg-blue-500 hover:text-white' : 'text-unifest-green-dark border border-unifest-green-dark'}"
        >
            Feedback
            {#if unreadFeedbackCount > 0}
                <span class="ml-2 bg-red-500 text-white px-2 py-0.5 rounded-full text-xs">
                    {unreadFeedbackCount}
                </span>
            {/if}
        </a>
    </div>
        {/if}
        
        <!-- Event selector - Always show if in admin mode or if we have active events -->
        {#if !noSelect && (events.length > 0)}
            <div class="flex items-center space-x-2 mb-4">
                <label for="event-selector" class="font-medium">Select Event:</label>
                <select 
                    id="event-selector" 
                    class="bg-white border border-gray-300 rounded px-3 py-1"
                    value={eventId || ''} 
                    on:change={(e) => changeEvent(e.target.value)}
                >
                    <option value="">Current/Upcoming Event</option>
                    {#each isAdmin ? events : activeEvents as event}
                        <option value={event.id}>{event.name}</option>
                    {/each}
                </select>
            </div>
        {/if}
        
        {#if isAdmin}
            <div class="flex items-center my-2">
                <label class="flex items-center">
                    <input type="checkbox" bind:checked={showPastMeals} class="mr-2">
                    Show past meals
                </label>
            </div>
        {/if}
    </header>

    <!-- Loading state -->
    {#if isLoading && days.length === 0}
        <div class="bg-unifest-green bg-opacity-20 p-4 rounded-md animate-pulse">
            <p>Loading...</p>
            <p class="text-sm text-gray-600">
                (If you see this for more than a second, there is probably something wrong 😕)
            </p>
        </div>
    {/if}

    <!-- Error state -->
    {#if error}
        <div class="bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded mb-4">
            <p class="font-bold">Error</p>
            <p>{error}</p>
        </div>
    {/if}

    <!-- Past event warning -->
    {#if isPastEvent && !error}
        <div class="bg-yellow-100 border border-yellow-400 text-yellow-700 px-4 py-3 rounded mb-4">
            <p class="font-bold">Note</p>
            <p>This is a past event. All meals have already been served.</p>
            {#if isAdmin}
                <p class="mt-2">As an admin, you can use the checkbox above to view past meals.</p>
            {/if}
        </div>
    {/if}

    <!-- Data display -->
    {#if days.length > 0}
        {#each days as day}
            {#if day.filter(meal => meal_filter(meal)).length > 0}
                <div class="mb-6">
                    <h2 class="text-xl font-semibold bg-unifest-green-dark text-white px-4 py-2 rounded-t-md">
                        {formatDate(day[0].status.start, {
                            weekday: 'long',
                            month: 'long',
                            day: 'numeric'
                        })}
                    </h2>
                    <div class="flex flex-col gap-2 p-2 bg-unifest-green bg-opacity-10 rounded-b-md">
                        {#each day as meal}
                            {#if meal_filter(meal)}
                                <Meal meal={meal.status}/>
                            {/if}
                        {/each}
                    </div>
                </div>
            {/if}
        {/each}
    {:else if !isLoading && !error}
        <div class="bg-yellow-100 border border-yellow-400 text-yellow-700 px-4 py-3 rounded">
            <p>No active meals available for this event.</p>
            {#if isAdmin}
                <p class="mt-2">Try toggling "Show past meals" if you're looking for completed meals.</p>
            {/if}
        </div>
    {/if}

    <!-- Feedback form section -->
    <section class="mt-12 p-6 border border-gray-300 rounded-md bg-gray-50">
        <h2 class="text-xl font-semibold mb-4">Send Us Your Feedback</h2>
        <p class="mb-2 text-gray-700">
            We appreciate your thoughts and suggestions to improve this page.
        </p>
        <textarea
            class="w-full p-3 border rounded resize-y focus:outline-unifest-green-dark focus:ring-2 focus:ring-unifest-green-dark"
            rows="4"
            placeholder="Write your feedback here..."
            bind:value={feedbackText}
        ></textarea>
        {#if feedbackStatus === 'error'}
            <p class="mt-2 text-red-600 font-semibold">{feedbackMessage}</p>
        {:else if feedbackStatus === 'success'}
            <p class="mt-2 text-green-600 font-semibold">{feedbackMessage}</p>
        {/if}
        <button
            on:click={submitFeedback}
            class="mt-4 bg-unifest-green-dark text-white font-semibold px-4 py-2 rounded hover:bg-unifest-green focus:outline-none focus:ring-2 focus:ring-unifest-green"
            type="button"
        >
            Submit Feedback
        </button>
    </section>
</div>
