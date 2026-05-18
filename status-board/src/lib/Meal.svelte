<script>
  export let meal;

  let eta_update_variants = [5, 10, 15, 30];
  let eta_add_variants = [1, 2, 5, 10];

  import { page } from '$app/stores';
  import { updateMealStatus, formatTime } from '$lib/api.ts';

  // German display labels for canonical allergen names. Anything not in the
  // map gets a capitalized fallback so unknown labels still read reasonably.
  const ALLERGEN_DE = {
    milch: 'Milch', gluten: 'Gluten', weizen: 'Weizen', eier: 'Eier',
    soja: 'Soja', erdnüsse: 'Erdnüsse', schalenfrüchte: 'Schalenfrüchte',
    sesamsamen: 'Sesam', sellerie: 'Sellerie', senf: 'Senf',
    lupine: 'Lupine', 'schwefeldioxid & sulfite': 'Sulfite', fisch: 'Fisch',
    schwein: 'Schwein', fleisch: 'Fleisch', milchprodukt: 'Milchprodukt',
    käse: 'Käse', 'ei-produkt': 'Ei-Produkt', krebstiere: 'Krebstiere',
    weichtiere: 'Weichtiere', gelatine: 'Gelatine',
  };

  function pretty(name) {
    if (name in ALLERGEN_DE) return ALLERGEN_DE[name];
    return name.charAt(0).toUpperCase() + name.slice(1);
  }

  // Split `meal.allergens` (an array of canonical names) into Contains and
  // MayContain lists. `~spuren-X` is the may-contain convention.
  $: contains = (meal.allergens ?? [])
    .filter(n => !n.startsWith('~spuren-'))
    .map(pretty)
    .sort();
  $: mayContain = (meal.allergens ?? [])
    .filter(n => n.startsWith('~spuren-'))
    .map(n => pretty(n.slice('~spuren-'.length)))
    .sort();
  $: dietary = meal.dietary ?? { vegan: false, vegetarian: false, lactose_free: false, gluten_free: false };

  // Fix admin mode to be case insensitive
  const adminPassword = $page.url.searchParams.get('admin');
  let isAdmin = adminPassword?.toUpperCase() === "TRUE";

  let etaInput = 0;
  let customMessageInput = meal.msg || '';

  function updateMeal() {
    meal.msg = customMessageInput;
    updateMealStatus(meal).catch(err => {
      console.error('Failed to update meal:', err);
      alert('Failed to update meal status');
    });
  }

  function setETA(minutes) {
    etaInput = minutes;

    // Convert minutes to seconds and add to current Unix timestamp
    const currentTimestamp = Date.now() / 1000 + (2 * 60 * 60); // Add 2 hours to match meal times
    meal.eta = currentTimestamp + (minutes * 60);
    meal.over = false;
    
    console.log(`Setting ETA for meal ${meal.meal_id} to ${minutes} minutes from now (timestamp: ${meal.eta})`);
    
    // Always update the meal after changing ETA
    updateMealStatus(meal).then(response => {
      console.log('ETA update successful:', response);
      
      // Locate and update our meal from the response
      const updatedMeal = response.find(m => m.meal_id === meal.meal_id);
      if (updatedMeal) {
        // Update our local copy with server values
        Object.assign(meal, updatedMeal);
        console.log(`Updated local state for meal ${meal.meal_id}:`, meal);
      }
    }).catch(err => {
      console.error('Failed to update meal ETA:', err);
      alert('Failed to update meal status');
    });
  }
  
  function addETA(minutes) {
    const currentTimestamp = Math.floor(Date.now() / 1000) + (2 * 60 * 60); // Add 2 hours to match meal times
    // Add minutes to the current time or existing ETA, whichever is later
    meal.eta = Math.max(meal.eta, currentTimestamp) + (minutes * 60);
    meal.over = false;
    
    console.log(`Adding ${minutes} minutes to ETA for meal ${meal.meal_id} (new timestamp: ${meal.eta})`);
    
    // Always update the meal after changing ETA
    updateMealStatus(meal).then(response => {
      console.log('ETA addition successful:', response);
      
      // Locate and update our meal from the response
      const updatedMeal = response.find(m => m.meal_id === meal.meal_id);
      if (updatedMeal) {
        // Update our local copy with server values
        Object.assign(meal, updatedMeal);
        console.log(`Updated local state for meal ${meal.meal_id}:`, meal);
      }
    }).catch(err => {
      console.error('Failed to add to meal ETA:', err);
      alert('Failed to update meal status');
    });
  }
  
  function endMeal() {
    meal.eta = 0;
    meal.over = true;
    updateMeal();
  }

  function nowServing() {
    meal.over = false;
    setETA(0);
  }

  // Use UTC time to avoid timezone issues
  let current_time = Math.floor(Date.now()/1000);
  // Calculate minutes until food, using the raw timestamp difference
  // to ensure timezone consistency
  let min_til_food = (meal.start - current_time)/60;
  
  // Set up timers - use UTC-consistent calculations
  setInterval(() => {
    current_time = Date.now()/1000;
    // add 2 hours to the current time to match the meal times
    current_time += 2 * 60 * 60; // 2 hours in seconds
    min_til_food = (Math.max(meal.start, meal.eta) - current_time)/60;
  }, 1000);
</script>

<div class="bg-white border-2 border-unifest-green rounded-md shadow-md p-4 mb-3">
  <div class="flex justify-between items-start">
    <div>
      <h3 class="text-lg font-semibold text-unifest-green-dark">
        {meal.recipe} {#if isAdmin}<span class="text-sm text-gray-500">({meal.meal_id})</span>{/if}
      </h3>
      <button 
        on:click={() => window.open(`http://catering.campus-kit.de/recipes/export_pdf/${meal.recipe_id}?energy=3000&number_of_servings=1`, '_blank')}
        class="bg-unifest-green text-white text-xs px-2 py-1 rounded hover:bg-unifest-green-dark"
      >
        Ingredients
      </button>
      <p class="text-sm text-gray-600 mb-2">Location: {meal.place}</p>
    </div>
    
    <div class="text-sm text-right">
      <span class="inline-block text-gray-600">
        {formatTime(meal.start)} - {formatTime(meal.end)}
      </span>
    </div>
  </div>
  
  {#if dietary.vegan || dietary.vegetarian || dietary.lactose_free || dietary.gluten_free || contains.length > 0 || mayContain.length > 0}
    <div class="mt-2 mb-1 text-sm">
      <div class="flex flex-wrap gap-1 mb-1">
        {#if dietary.vegan}
          <span class="px-2 py-0.5 rounded-full text-xs font-semibold bg-green-100 text-green-800 border border-green-300">vegan</span>
        {:else if dietary.vegetarian}
          <span class="px-2 py-0.5 rounded-full text-xs font-semibold bg-lime-100 text-lime-800 border border-lime-300">vegetarisch</span>
        {/if}
        {#if dietary.lactose_free}
          <span class="px-2 py-0.5 rounded-full text-xs font-semibold bg-blue-100 text-blue-800 border border-blue-300">laktosefrei</span>
        {/if}
        {#if dietary.gluten_free}
          <span class="px-2 py-0.5 rounded-full text-xs font-semibold bg-purple-100 text-purple-800 border border-purple-300">glutenfrei</span>
        {/if}
      </div>
      {#if contains.length > 0}
        <p class="text-gray-700">
          <span class="font-semibold">Enthält:</span> {contains.join(', ')}
        </p>
      {/if}
      {#if mayContain.length > 0}
        <p class="text-xs italic text-gray-500">
          Kann Spuren von {mayContain.join(', ')} enthalten.
        </p>
      {/if}
    </div>
  {/if}

  <div class="mt-2">
    <div class="flex items-center mt-1">
      <span class="text-base font-medium">Status:</span>
      <div class="ml-2 px-3 py-1 rounded-full {meal.over || meal.end < current_time ? 'bg-red-100 text-red-700' : meal.start > current_time ? 'bg-blue-100 text-blue-700' : meal.eta <= current_time && meal.end >= current_time ? 'bg-green-100 text-green-700' : 'bg-yellow-100 text-yellow-700'}">
        {#if meal.over || meal.end < current_time}
          <span class="flex items-center">
            <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 mr-1" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
            </svg>
            Finished
          </span>
        {:else if meal.eta > current_time && meal.eta >= 0}
          <span class="flex items-center">
            <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 mr-1" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
            </svg>
            Coming in {Math.floor((meal.eta - current_time) /60)} min {Math.floor(((meal.eta - current_time)/60 % 1) * 60)} sec
          </span>
        {:else if meal.start > current_time}
          <span class="flex items-center">
            <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 mr-1" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
            </svg>
            Upcoming 
            {#if min_til_food < 5 && min_til_food > 0}
              (Starting in {Math.floor(min_til_food)} min {Math.floor((min_til_food % 1) * 60)} sec)
            {:else if min_til_food < 60 && min_til_food > 0}
              (Starting in {Math.floor(min_til_food)} min)
            {:else}
              (Starting at {formatTime(meal.start)})
            {/if}
          </span>
        {:else if meal.eta <= current_time && meal.end >= current_time}
          <span class="flex items-center">
            <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 mr-1" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
            </svg>
            Now Serving
          </span>
        {/if}
      </div>
    </div>
    
    {#if meal.msg}
      <div class="mt-2 p-2 bg-gray-100 rounded">
        <p class="text-gray-800">{meal.msg}</p>
      </div>
    {/if}
  </div>

  {#if isAdmin}
    <div class="mt-4 pt-3 border-t border-gray-200">
      <h4 class="font-semibold mb-2">Admin Controls</h4>
      
      <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
        <div>
          <label class="block text-sm font-medium text-gray-700 mb-1">Set ETA (minutes):</label>
          <div class="flex flex-wrap items-center gap-2">
            <input 
              type="number" 
              bind:value={etaInput}
              class="w-16 border rounded px-2 py-1 text-sm" 
            >
            <button 
              on:click={() => {setETA(etaInput)}}
              class="bg-unifest-green-dark text-white text-sm px-2 py-1 rounded"
            >
              Set
            </button>
          </div>
          
          <div class="flex flex-wrap gap-1 mt-2">
            {#each eta_update_variants as label}
              <button 
                on:click={() => {setETA(label)}}
                class="bg-unifest-green text-unifest-green-dark text-xs px-2 py-1 rounded"
              >
                {label}min
              </button>
            {/each}
          </div>
          
          <div class="flex flex-wrap gap-1 mt-2">
            {#each eta_add_variants as label}
              <button 
                on:click={() => {addETA(label)}}
                class="bg-blue-100 text-blue-700 text-xs px-2 py-1 rounded"
              >
                +{label}min
              </button>
            {/each}
          </div>
        </div>
        
        <div>
          <label class="block text-sm font-medium text-gray-700 mb-1">Custom Message:</label>
          <textarea
            bind:value={customMessageInput}
            class="w-full border rounded px-2 py-1 text-sm h-20"
          ></textarea>
          
          <div class="flex justify-between mt-2">
            <button 
              on:click={updateMeal}
              class="bg-unifest-green-dark text-white text-sm px-3 py-1 rounded"
            >
              Update Message
            </button>
            
            <button 
              on:click={nowServing}
              class="bg-green-600 text-white text-sm px-3 py-1 rounded"
            >
              Now Serving
            </button>
            
            <button 
              on:click={endMeal}
              class="bg-red-600 text-white text-sm px-3 py-1 rounded"
            >
              End Meal
            </button>
          </div>
        </div>
      </div>
    </div>
  {/if}
</div>
