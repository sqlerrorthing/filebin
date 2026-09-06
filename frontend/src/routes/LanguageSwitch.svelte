<script lang="ts">
    import {setLocale, getLocale, locales} from '$lib/paraglide/runtime';
    import {cn} from "$lib/utils";
    import * as m from "$lib/paraglide/messages";

    const currentLocale = getLocale();
</script>

<div class="flex flex-wrap items-center justify-center">
    {#each locales as l}
        {@const region = new Intl.Locale(l).region?.toLowerCase() || l}
        <button
                class={cn(
                    "p-0.5 border-transparent border border-dotted cursor-pointer",
                    currentLocale === l && "border-foreground"
                )}
                onclick={() => setLocale(l)}
                title={m.language()}
        >
            <img src="/flags/{region}.svg" alt={l} class="w-5 h-4 object-cover select-none pointer-events-none"/>
        </button>
    {/each}
</div>
