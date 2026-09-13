<script lang="ts">
    import { Globe, Moon, Sun } from "@lucide/svelte";
    import icon from "$lib/assets/icon_32x32.png";
    import LangSelectDropdown from "./LangSelectDropdown.svelte";
    import { localizeHref } from "$lib/paraglide/runtime";
    import { toggleMode } from "mode-watcher";
    import NavButton from "$lib/components/header/NavButton.svelte";

    let isLangMenuOpen = $state(false);

    function toggleLangMenu() {
        isLangMenuOpen = !isLangMenuOpen;
    }

    function handleClickOutside(event: MouseEvent) {
        const target = event.target as HTMLElement;
        if (!target.closest(".lang-dropdown")) {
            isLangMenuOpen = false;
        }
    }
</script>

<svelte:window onclick={handleClickOutside} />

<div
    class="mx-auto flex h-full max-w-7xl items-center justify-between px-4
        md:px-6"
>
    <a href={localizeHref("/")} class="flex items-center gap-2">
        <div class="bg-background flex size-9 items-center justify-center">
            <img src={icon} alt="" />
        </div>

        <span class="text-lg font-semibold">
            filebin<span class="text-muted-foreground">.lol</span>
        </span>
    </a>

    <div class="flex items-center gap-1">
        <div class="lang-dropdown relative">
            <NavButton onclick={toggleLangMenu} aria-label="Select language">
                <Globe class="size-4" />
            </NavButton>

            {#if isLangMenuOpen}
                <LangSelectDropdown />
            {/if}
        </div>

        <NavButton onclick={toggleMode}>
            <Sun class="size-4 dark:hidden" />
            <Moon class="hidden size-4 dark:block" />
        </NavButton>
    </div>
</div>
