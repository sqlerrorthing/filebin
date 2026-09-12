<script lang="ts">
    import "./layout.css";
    import favicon from "$lib/assets/favicon.ico";
    import { Globe, Moon, Sun } from "@lucide/svelte";
    import NavButton from "./NavButton.svelte";
    import { ModeWatcher, toggleMode } from "mode-watcher";
    import icon from "$lib/assets/icon_32x32.png";
    import {getLocale, locales, setLocale} from "$lib/paraglide/runtime";

    let { children } = $props();

    const currentLocale = getLocale();

    let isLangMenuOpen = $state(false);

    function toggleLangMenu() {
        isLangMenuOpen = !isLangMenuOpen;
    }

    function changeLanguage(lang: typeof locales[number]) {
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
    <header class="h-16 border-b border-border">
        <div class="mx-auto flex h-full max-w-7xl items-center justify-between px-4 md:px-6">
            <a href="/" class="flex items-center gap-2">
                <div class="flex size-9 items-center justify-center bg-background">
                    <img src={icon} alt="" />
                </div>

                <span class="font-semibold text-lg">
                    filebin<span class="text-muted-foreground">.lol</span>
                </span>
            </a>

            <div class="flex items-center gap-1">
                <div class="relative lang-dropdown">
                    <NavButton onclick={toggleLangMenu} aria-label="Select language">
                        <Globe class="size-4" />
                    </NavButton>

                    {#if isLangMenuOpen}
                        <div class="absolute right-0 mt-2 w-32 rounded-md border border-border bg-popover p-1 shadow-md z-50">
                            {#each locales as lang}
                                <button
                                        type="button"
                                        onclick={() => changeLanguage(lang)}
                                        class="flex w-full items-center justify-between rounded-sm px-3 py-1.5 text-sm transition-colors hover:bg-accent hover:text-accent-foreground {lang === currentLocale ? 'font-bold text-primary' : ''}"
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

    <main class="min-h-[calc(100vh-4rem)] flex flex-col">
        {@render children()}
    </main>
</div>