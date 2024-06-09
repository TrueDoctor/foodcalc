<script>
    import { page } from '$app/stores'
    import Meal from '$lib/Meal.svelte'
    const adminPassword = $page.url.searchParams.get('admin')
    let isAdmin =  adminPassword == "TEST";

    let status = { }
    let promise = fetch('https://essen.campus-kit.de/api/').then((x) => x.json());
    setInterval(() => promise = fetch('https://essen.campus-kit.de/api/').then((x) => x.json()), 1000*60*0.5);
</script>

{#if isAdmin} 
  <h1>ADMIN MODE</h1>
{/if}

{#await promise}
  Loading...
  (If you see this for more than a second, there is probably something wrong :0)
{:then days}
    {#each days as day}
      <h1> {new Date(day[0].status.start*1000).toLocaleDateString('de-DE', {
        weekday: 'long',
        month: 'long',
        day: 'numeric',
      })} </h1>
      <div class="flex flex-col gap-2 p-2">
        {#each day as meal}
          {#if ((meal.status.end - Date.now()/1000)/60) > (-3*60)}
            <Meal meal={meal}/>
          {/if}
        {/each}
      </div>
    {/each}
{:catch error}
   Sorry, there was an error :(
   <script> console.log(error) </script>
{/await}

