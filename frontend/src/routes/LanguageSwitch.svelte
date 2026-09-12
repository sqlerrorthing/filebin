<script lang="ts">
    import { setLocale, getLocale, locales } from "$lib/paraglide/runtime";
    import { cn } from "$lib/utils";
    import * as m from "$lib/paraglide/messages";

    const currentLocale = getLocale();
</script>

<div class="flex flex-wrap items-center justify-center">
    {#each locales as l}
        {@const region = new Intl.Locale(l).region?.toLowerCase() || l}
        <button
            class={cn(
                "cursor-pointer border border-dotted border-transparent p-0.5",
                currentLocale === l && "border-foreground"
            )}
            onclick={() => setLocale(l)}
            title={m["display-language"](undefined, { locale: l })}
        >
            <img
                src="/lang/{region}.svg"
                alt={l}
                class="pointer-events-none h-4 w-5 object-cover select-none"
            />
        </button>
    {/each}
</div>
