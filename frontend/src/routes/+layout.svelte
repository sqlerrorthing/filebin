<script lang="ts">
    import "./layout.css";
    import favicon from "$lib/assets/favicon.ico";
    import { Globe, Moon, Sun } from "@lucide/svelte";
    import NavButton from "./NavButton.svelte";
    import { ModeWatcher, toggleMode } from "mode-watcher";
    import icon from "$lib/assets/icon_32x32.png";
    import { getLocale, locales, setLocale } from "$lib/paraglide/runtime";

    let { children } = $props();

    const currentLocale = getLocale();

    let isLangMenuOpen = $state(false);

    function toggleLangMenu() {
        isLangMenuOpen = !isLangMenuOpen;
    }

    function changeLanguage(lang: (typeof locales)[number]) {
        setLocale(lang);
        isLangMenuOpen = false;
    }

    function handleClickOutside(event: MouseEvent) {
        const target = event.target as HTMLElement;
        if (!target.closest(".lang-dropdown")) {
            isLangMenuOpen = false;
        }
    }
</script>

<svelte:window onclick={handleClickOutside} />

<svelte:head>
    <link rel="icon" href={favicon} />
</svelte:head>

<ModeWatcher />

<div class="bg-background text-foreground min-h-screen">
    <header class="border-border h-16 border-b">
        <div
            class="mx-auto flex h-full max-w-7xl items-center justify-between
                px-4 md:px-6"
        >
            <a href="/" class="flex items-center gap-2">
                <div
                    class="bg-background flex size-9 items-center
                        justify-center"
                >
                    <img src={icon} alt="" />
                </div>

                <span class="text-lg font-semibold">
                    filebin<span class="text-muted-foreground">.lol</span>
                </span>
            </a>

            <div class="flex items-center gap-1">
                <div class="lang-dropdown relative">
                    <NavButton
                        onclick={toggleLangMenu}
                        aria-label="Select language"
                    >
                        <Globe class="size-4" />
                    </NavButton>

                    {#if isLangMenuOpen}
                        <div
                            class="border-border bg-popover absolute right-0
                                z-50 mt-2 w-32 rounded-md border p-1 shadow-md"
                        >
                            {#each locales as lang}
                                <button
                                    type="button"
                                    onclick={() => changeLanguage(lang)}
                                    class="hover:bg-accent
                                        hover:text-accent-foreground flex w-full
                                        items-center justify-between rounded-sm
                                        px-3 py-1.5 text-sm transition-colors{lang ===
                                    currentLocale
                                        ? "text-primary font-bold"
                                        : ""}"
                                >
                                    <span class="uppercase">{lang}</span>
                                    {#if lang === currentLocale}
                                        <span class="text-xs">✓</span>
                                    {/if}
                                </button>
                            {/each}
                        </div>
                    {/if}
                </div>

                <NavButton onclick={toggleMode}>
                    <Sun class="size-4 dark:hidden" />
                    <Moon class="hidden size-4 dark:block" />
                </NavButton>
            </div>
        </div>
    </header>

    <main class="flex min-h-[calc(100vh-4rem)] flex-col">
        {@render children()}
    </main>
</div>
