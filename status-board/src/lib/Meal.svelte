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
  //let eta_date = new Date(absolute_eta);
  let start_date = new Date(meal.status.start*1000);
  let end_date = new Date(meal.status.end*1000);
</script>
<div class="bg-unifest-green m-5 p-3 rounded-md">
  <p> "{meal.status.recipe}" ({meal.meal_id}) </p>
  <p> {start_date.toLocaleString('de-DE', optionsTime)}
    - {end_date.toLocaleString('de-DE', optionsTime)} </p>
  {#if !isAdmin}
    <p> Status: 
      {#if meal.status.eta == 0 || (absolute_eta < Date.now() && meal.status.eta >= 0)}
        ✅ Serving 
      {:else if meal.status.eta > 0} 
        <p>⚠️ More is on the way, comming in about {Math.ceil((absolute_eta - Date.now())/1000/60)}min </p>
        <p> {new Date(absolute_eta).toLocaleString('de-DE', optionsDate)} </p>
      {:else if meal.status.eta < 0} 
        ❌ Sorry, this meal has finished serving :/
      {/if}
    </p>
  {:else}
    <p> ETA: <p> <input type="number" bind:value={meal.status.eta}>
      {#each eta_update_variants as label}
        <button on:click={() => {setETA(label)}}>Set ETA to {label}min </button>
      {/each}
      <button on:click={() => {setETA(-1)}}>Meal is over </button>
    <p> Custom Message: <p> <input type="text" bind:value={meal.status.msg}>
    <button on:click={updateMeal}> UPDATE! </button>
  {/if}
</div>
