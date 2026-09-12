<script lang="ts">
    import * as m from "$lib/paraglide/messages";
    import {cn} from "$lib/utils";
    import type {Component} from "svelte";
    import type {LucideProps} from "@lucide/svelte";

    let {
        name,
        description,
        highlight = false,
        icon: Icon
    }: {
        name: () => string;
        description: () => string;
        highlight?: boolean;
        icon?: Component<LucideProps>
    } = $props();
</script>

<button class={cn(
    "w-full md:w-84 rounded-xl flex flex-col p-4 gap-2 items-start text-start cursor-pointer",
    "transition-colors duration-150",
    highlight
        ? "bg-accent-foreground text-accent hover:bg-accent-foreground/85"
        : "bg-background hover:bg-accent/85"
)} title={name()}>
    <span class="font-bold text-xl flex items-center justify-between w-full">
        {name()}

        {#if Icon}
            <Icon class="w-4 h-4"></Icon>
        {/if}
    </span>

    <span class={cn(
        highlight ? "text-muted" : "text-muted-foreground"
    )}>
        {description()}
    </span>
</button>
