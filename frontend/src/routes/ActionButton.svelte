<script lang="ts">
    import * as m from "$lib/paraglide/messages";
    import { cn } from "$lib/utils";
    import type { Component } from "svelte";
    import type { LucideProps } from "@lucide/svelte";

    let {
        name,
        description,
        highlight = false,
        icon: Icon,
    }: {
        name: () => string;
        description: () => string;
        highlight?: boolean;
        icon?: Component<LucideProps>;
    } = $props();
</script>

<button
    class={cn(
        `flex w-full cursor-pointer flex-col items-start gap-2 rounded-xl p-4
        text-start md:w-84`,
        "transition-colors duration-150",
        highlight
            ? "bg-accent-foreground text-accent hover:bg-accent-foreground/85"
            : "bg-background hover:bg-accent/85"
    )}
    title={name()}
>
    <span class="flex w-full items-center justify-between text-xl font-bold">
        {name()}

        {#if Icon}
            <Icon class="h-4 w-4"></Icon>
        {/if}
    </span>

    <span class={cn(highlight ? "text-muted" : "text-muted-foreground")}>
        {description()}
    </span>
</button>
