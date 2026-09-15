<script lang="ts">
    import { getFilesContext } from "$lib/context/files.svelte";
    import { Home, ChevronRight } from "@lucide/svelte";

    const filesCtx = getFilesContext();
</script>

<nav class="flex items-center space-x-2 text-sm text-muted-foreground overflow-x-auto py-2">
    <button
        onclick={() => (filesCtx.currentPath = [])}
        class="flex items-center gap-1 hover:text-foreground transition-colors shrink-0 cursor-pointer"
    >
        <Home class="size-4" />
        <span>Root</span>
    </button>
    {#each filesCtx.currentPath as segment, index}
        <ChevronRight class="size-4 shrink-0" />
        <button
            onclick={() => (filesCtx.currentPath = filesCtx.currentPath.slice(0, index + 1))}
            class="hover:text-foreground transition-colors shrink-0 font-medium text-foreground cursor-pointer"
        >
            {segment}
        </button>
    {/each}
</nav>
