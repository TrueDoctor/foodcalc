<script>
    import { page } from '$app/stores'
    import Meal from '$lib/Meal.svelte'
    const adminPassword = $page.url.searchParams.get('admin')
    let isAdmin =  adminPassword == "TEST";

    let status = { }
</script>

{#if isAdmin} 
  <h1>ADMIN MODE</h1>
  <p> Falls du hier ausversehen 
{/if}

{#await fetch('https://essen.campus-kit.de/api/').then((x) => x.json())}
  Loading...
  (If you see this for more than a second, there is probably something wrong :0)
{:then meals.sort((/** @type {{ name: string; }} */ a, /** @type {{ name: any; }} */ b) => a.name.localeCompare(b.name)) as meal}
  <div class="grid grid-flow-row grid-cols-4">
    {#each meals as meal}
      <Meal meal={meal}/>
    {/each}
  </div>
{:catch error}
   Sorry, there was an error :(
   <script> console.log(error) </script>
{/await}

