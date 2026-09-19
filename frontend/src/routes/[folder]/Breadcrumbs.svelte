<script lang="ts">
    import {getFilesContext} from "$lib/context/files.svelte";
    import {ChevronRight, Home} from "@lucide/svelte";

    const ctx = getFilesContext();

    const path = $derived(ctx.path);
</script>

<nav aria-label="Breadcrumb" class="flex items-center gap-2 mt-3 text-sm text-muted-foreground">
    {#if ctx.isRoot}
        <div class="flex items-center gap-1.5 font-medium text-foreground">
            <Home class="h-4 w-4" />
            <span>Root</span>
        </div>
    {:else}
        <button
            type="button"
            onclick={() => ctx.resetPath()}
            class="flex items-center gap-1.5 hover:text-foreground transition-colors cursor-pointer"
        >
            <Home class="h-4 w-4" />
            <span>Root</span>
        </button>

        {#each path as segment, index}
            <ChevronRight class="h-3.5 w-3.5 text-muted-foreground/60" />
            {#if index === path.length - 1}
                <span class="font-medium text-foreground">{segment}</span>
            {:else}
                <button
                    type="button"
                    onclick={() => {
                        const diff = path.length - 1 - index;
                        for (let i = 0; i < diff; i++) {
                            ctx.goUp();
                        }
                    }}
                    class="hover:text-foreground transition-colors cursor-pointer"
                >
                    {segment}
                </button>
            {/if}
        {/each}
    {/if}
</nav>
