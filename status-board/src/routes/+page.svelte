<script>
    import { page } from '$app/stores'
    import Meal from '$lib/Meal.svelte'
    import { onMount } from 'svelte';

    let days = [];
    onMount(() => {
      async function fetchData() {
        let data = await fetch('https://essen.campus-kit.de/api/');
        days = await data.json();
        console.log(days);
      }
    
      const interval = setInterval(fetchData, 1000*10);
      fetchData();

      return () => clearInterval(interval);
    });
    
    const adminPassword = $page.url.searchParams.get('admin')
    let isAdmin =  adminPassword == "TEST";

    let meal_filter = (meal) => {return (meal.status.end + 3 * 3600 > Date.now()/1000)}
    $: if (days.length > 0) console.log(days[1].filter(meal_filter))
</script>

{#if isAdmin} 
  <h1>ADMIN MODE</h1>
{/if}

{#if !days}
  Loading...
  (If you see this for more than a second, there is probably something wrong :0)
{:else }
    {#each days as day}
      {#if day.filter(meal_filter).length > 0}
        <h1> {new Date(day[0].status.start*1000).toLocaleDateString('de-DE', {
          weekday: 'long',
          month: 'long',
          day: 'numeric',
        })} </h1>
        <div class="flex flex-col gap-2 p-2">
          {#each day as meal}
            {#if meal_filter(meal)}
              <Meal meal={meal.status}/>
            {/if}
          {/each}
        </div>
      {/if}
    {/each}
{/if}

