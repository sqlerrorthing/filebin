<script lang="ts">
    import {cn} from "$lib/utils";
    import {getLocale, locales, setLocale} from "$lib/paraglide/runtime";
    import { Check } from "@lucide/svelte";
    import * as m from "$lib/paraglide/messages";

    const currentLocale = getLocale();

    function changeLanguage(lang: (typeof locales)[number]) {
        setLocale(lang);
    }
</script>

<div class="border-border bg-popover absolute right-0
        z-50 mt-2 w-64 rounded-md border p-1 shadow-md"
>
    {#each locales as lang}
        <button
                type="button"
                onclick={() => changeLanguage(lang)}
                class={cn(`
                    hover:bg-accent
                    hover:text-accent-foreground flex gap-2 w-full
                    rounded-sm items-center
                    px-3 py-1.5 text-sm transition-colors`,
                    currentLocale && "text-primary font-bold"
                )}
        >
            <img
                src="/lang/{lang}.svg"
                alt={lang}
                class="pointer-events-none h-4 w-5 object-cover select-none"
            />

            <span>
                {m[`languages.${lang}`]({}, { locale: lang })}
                <span class="italic text-muted-foreground ml-2 font-thin">
                    {m[`languages.${lang}`]()}
                </span>
            </span>

            {#if lang === currentLocale}
                <Check class="w-3 h-auto ml-auto" />
            {/if}
        </button>
    {/each}
</div>
