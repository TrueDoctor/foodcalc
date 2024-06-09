<script>
    import { page } from '$app/stores'
    import Meal from '$lib/Meal.svelte'
    import { onMount } from 'svelte';

    let days = [];
    onMount(() => {
      async function fetchData() {
        let data = await fetch('https://essen.campus-kit.de/api/');
        days = await data.json();
      }
    
      const interval = setInterval(fetchData, 1000);
      fetchData();

      return () => clearInterval(interval);
    });
    
    const adminPassword = $page.url.searchParams.get('admin')
    let isAdmin =  adminPassword == "TEST";

    let status = { }
</script>

{#if isAdmin} 
  <h1>ADMIN MODE</h1>
{/if}

{#if !days}
  Loading...
  (If you see this for more than a second, there is probably something wrong :0)
{:else }
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
{/if}

