<script>
  export let meal;

  let eta_update_variants = [5, 10];
  const optionsTime = {
    hour: 'numeric',
    minute: 'numeric'
  };
  const optionsDate = {
    weekday: 'long',
    year: 'numeric',
    month: 'long',
    day: 'numeric',
    hour: 'numeric',
    minute: 'numeric'
  };
  import { page } from '$app/stores'
  const adminPassword = $page.url.searchParams.get('admin')
  let isAdmin =  adminPassword == "TEST";

  function updateMeal() {
    fetch(`https://essen.campus-kit.de/api/${meal.meal_id}`, {
    		method: 'POST',
        headers: {
          "Content-Type": "application/json",
        },
    		body: JSON.stringify(meal.status)
    	})
  }

  function setETA(minutes){
    meal.status.eta = minutes;
    updateMeal()
  }

  let absolute_eta = (meal.status.last_modified + meal.status.eta * 60)*1000;
  let start_date = new Date(meal.status.start*1000);
  let end_date = new Date(meal.status.end*1000);
  let min_til_food = (meal.status.start - Date.now()/1000)/60;
  setInterval(() => min_til_food = (meal.status.start - Date.now()/1000)/60, 1)
  console.log(meal.status.end)
</script>
<div class="bg-unifest-green p-3 rounded-md">
  <p> "{meal.status.recipe}" ({meal.meal_id}) 
    <!-- <p> {start_date.toLocaleString('de-DE', optionsTime)}
      - {end_date.toLocaleString('de-DE', optionsTime)} </p> -->
  </p>
  {#if !isAdmin}
    <p> Status: 
      {#if meal.status.start < Date.now()}
        🕒 Upcoming 
        {#if min_til_food < 5}
          (Starting in {Math.floor(min_til_food)} min {Math.floor((min_til_food % 1) * 60)} sec )
        {:else if min_til_food < 60}
          (Starting in {Math.floor(min_til_food)} min )
        {:else}
          (Starting at {start_date.toLocaleString('de-DE', optionsTime)})
        {/if}
      {:else if meal.status.eta == 0 || (absolute_eta < Date.now() && meal.status.eta >= 0)}
        ✅ Serving 
      {:else if meal.status.eta > 0} 
        <p>⚠️ More is on the way, comming in about {Math.floor(absolute_eta/60)} min {Math.floor((absolute_eta/60 % 1) * 60)} sec </p>
      {:else if meal.status.eta < 0} 
        ❌ Sorry, this meal has finished serving :/
      {/if}
    </p>
  {:else}
    <p> ETA: <p> <input type="number" bind:value={meal.status.eta}>
      {#each eta_update_variants as label}
        <button on:click={() => {setETA(label)}}>Set ETA to {label}min </button>
      {/each}
      <button on:click={() => {setETA(-1)}}>Meal is over</button>
      <button on:click={() => {setETA(0)}}>Now Serving</button>
    <p> Custom Message:  
      <input type="text" bind:value={meal.status.msg}>
      <button on:click={updateMeal}> UPDATE! </button>
    <p>
  {/if}
</div>
