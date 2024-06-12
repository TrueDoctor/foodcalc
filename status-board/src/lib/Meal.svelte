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
  let new_label = 0;

  function updateMeal() {
    fetch(`https://essen.campus-kit.de/api/${meal.meal_id}`, {
    		method: 'POST',
        headers: {
          "Content-Type": "application/json",
        },
    		body: JSON.stringify(meal)
    	})
  }

  function setETA(minutes){
    new_label = minutes;

    meal.eta = Math.ceil(minutes * 60 + Date.now() / 1000);
    updateMeal()
  }
  
  function endMeal(){

    meal.end = Math.ceil(Date.now() / 1000);
    updateMeal()
  }

  let absolute_eta = meal.eta;
  let start_date = new Date(meal.start*1000);
  let end_date = new Date(meal.end*1000);
  let min_til_food = (meal.start - Date.now()/1000)/60;
  let current_time = Date.now()/1000;
  setInterval(() => min_til_food = (meal.start - Date.now()/1000)/60, 1)
  setInterval(() => current_time = Date.now()/1000, 1)
  console.log(meal.end)
  console.log(meal);
</script>
<div class="bg-unifest-green p-3 rounded-md">
  <p> {meal.recipe} {#if isAdmin} ({meal.meal_id}) {/if} ({meal.place}) </p>
  {#if !isAdmin}
    <p> Status: 
      {#if meal.start > Date.now() / 1000}
        🕒 Upcoming 
        {#if min_til_food < 5 && min_til_food > 0}
          (Starting in {Math.floor(min_til_food)} min {Math.floor((min_til_food % 1) * 60)} sec )
        {:else if min_til_food < 60 && min_til_food > 0}
          (Starting in {Math.floor(min_til_food)} min )
        {:else}
          (Starting at {start_date.toLocaleString('de-DE', optionsTime)})
        {/if}
      {:else if absolute_eta <= Date.now() / 1000 }
        ✅ Serving 
      {:else if absolute_eta > Date.now() / 1000  && meal.eta >= 0} 
        <p>⚠️ More is on the way, comming in about {Math.floor((absolute_eta - current_time) /60)} min {Math.floor(((absolute_eta - current_time)/60 % 1) * 60)} sec </p>
      {:else if meal.eta < 0} 
        ❌ Sorry, this meal has finished serving :/
      {/if}
    </p>
  {:else}
    <p> ETA: <p> <input type="number" bind:value={new_label}>
      {#each eta_update_variants as label}
        <button on:click={() => {setETA(label)}}>Set ETA to {label}min </button>
      {/each}
      <button on:click={() => {endMeal()}}>Meal is over</button>
      <button on:click={() => {setETA(0)}}>Now Serving</button>
    <p> Custom Message:  
      <input type="text" bind:value={meal.msg}>
      <button on:click={updateMeal}> UPDATE! </button>
    <p>
  {/if}
</div>
