<script>
  export let meal;

  let eta_update_variants = [5, 10];
  let eta_add_variants = [1, 2, 5, 10];
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

  let etaInput = 0;
  let customMessageInput = '';

  function updateMeal() {
    meal.msg = customMessageInput; 
    fetch(`https://essen.campus-kit.de/api/${meal.meal_id}`, {
    		method: 'POST',
        headers: {
          "Content-Type": "application/json",
        },
    		body: JSON.stringify(meal)
    	})
  }

  function setETA(minutes){
    etaInput = minutes;

    meal.msg = customMessageInput; 
    meal.eta = Math.ceil(minutes * 60 + Date.now() / 1000);
    meal.over = false;
    updateMeal()
  }
  function addETA(minutes){
    meal.msg = customMessageInput; 
    meal.eta = Math.ceil(Math.max(meal.eta, Date.now() / 1000) +  minutes * 60);
    meal.over = false;
    updateMeal()
  }
  
  function endMeal(){

    //meal.end = Math.ceil(Date.now() / 1000);
    meal.eta = 0;
    meal.over = true;
    updateMeal()
  }

  function nowServing(){
    meal.over = false;
    setETA(0);
  }

  let start_date = new Date(meal.start*1000);
  let end_date = new Date(meal.end*1000);
  let current_time = Date.now()/1000;
  let min_til_food = (meal.start - current_time)/60;
  setInterval(() => min_til_food = (Math.max(meal.start, meal.eta) - Date.now()/1000)/60, 1)
  setInterval(() => current_time = Date.now()/1000, 1)
</script>
<div class="bg-unifest-green p-3 rounded-md">
  <p> {meal.recipe} {#if isAdmin} ({meal.meal_id}) {/if} ({meal.place}) </p>
  <p> {#if isAdmin} Public: {/if} Status: 
      {#if meal.start > current_time}
        🕒 Upcoming 
        {#if min_til_food < 5 && min_til_food > 0}
          (Starting in {Math.floor(min_til_food)} min {Math.floor((min_til_food % 1) * 60)} sec )
        {:else if min_til_food < 60 && min_til_food > 0}
          (Starting in {Math.floor(min_til_food)} min )
        {:else}
          (Starting at {start_date.toLocaleString('de-DE', optionsTime)})
        {/if}
      {:else if meal.eta <= current_time && meal.end >= current_time }
        ✅ Serving 
      {:else if meal.eta > current_time  && meal.eta >= 0} 
        <p>⚠️ More is on the way, comming in about {Math.floor((meal.eta - current_time) /60)} min {Math.floor(((meal.eta - current_time)/60 % 1) * 60)} sec </p>
      {:else if meal.end < current_time} 
        ❌ Sorry, this meal has finished serving :/
      {/if}
    {#if meal.msg != null }
      <p> {meal.msg} </p>
    {/if}
    </p>
  {#if isAdmin}
    <hr style="padding:3px" />
    <p> ETA:
      <input type="number" bind:value={etaInput}> <br>
      {#each eta_update_variants as label}
        <button on:click={() => {setETA(label)}}>{label}min </button>
      {/each}
      {#each eta_add_variants as label}
        <button on:click={() => {addETA(label)}}>+ {label}min</button>
      {/each} <br>
      <button on:click={() => {endMeal()}}>Meal is over</button> <br>
      <button on:click={() => {setETA(0)}}>Now Serving</button> <br>
    <p> Custom Message:  
      <input type="text" bind:value={customMessageInput}>
      <button on:click={updateMeal}> UPDATE! </button>
    <p>
  {/if}
</div>
