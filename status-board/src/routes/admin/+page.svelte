{#await fetch('http://10.1.1.4:8090').then((x) => x.json())}
  Loading...
  (If you see this for more than a second, there is probably something wrong :0)
{:then meals}
  <div class="grid grid-flow-row grid-cols-4">
    {#each meals as meal}
      <div class="bg-blue-300 m-5 p-1">
        Meal: {meal.status.recipe} ({meal.meal_id}) <br>
        Status: {#if meal.status.eta <= 0} Serving {:else} More is on the way, ETA: +{meal.status.eta}min {/if}
      </div>
    {/each}
  </div>
{:catch error}
   Sorry, there was an error :(
   <script> console.log(error) </script>
{/await}
