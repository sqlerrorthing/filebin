<script lang="ts">
    import { cn } from "$lib/utils";
    import type { Component } from "svelte";
    import type { LucideProps } from "@lucide/svelte";
    import type {HTMLButtonAttributes} from "svelte/elements";
    import type { Snippet } from "svelte";

    interface Props extends HTMLButtonAttributes {
        displayName: () => string;
        description: () => string;
        highlight?: boolean;
        icon?: Snippet;
    }

    let {
        displayName,
        description,
        highlight = false,
        icon,
        class: className, ...restProps
    }: Props = $props();
</script>

<button
    class={cn(
        `flex w-full min-h-29 cursor-pointer flex-col items-start gap-2 rounded-xl p-4
        text-start md:w-84`,
        "transition-colors duration-150",
        highlight
            ? "bg-accent-foreground text-accent hover:bg-accent-foreground/85"
            : "bg-background hover:bg-accent/85",
        className
    )}
    title={displayName()}
    {...restProps}
>
    <span class="flex w-full items-center justify-between text-xl font-bold">
        {displayName()}

        {#if icon}
            <span class="w-4 h-4 flex items-center justify-center shrink-0">
                {@render icon()}
            </span>
        {/if}
    </span>

    <span class={cn(highlight ? "text-muted" : "text-muted-foreground")}>
        {description()}
    </span>
</button>
