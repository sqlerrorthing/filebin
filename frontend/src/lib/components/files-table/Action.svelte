<script lang="ts">
    import type {HTMLButtonAttributes} from "svelte/elements";
    import {cn} from "$lib/utils";
    import type {Snippet} from "svelte";
    import {stopPropagation} from "svelte/legacy";

    interface Props extends HTMLButtonAttributes {
        children?: Snippet
    }

    let {
        children,
        onclick,
        class: className,
        disabled,
        ...restProps
    }: Props = $props();
</script>

<button type="button" onclick={(e) => {e.stopPropagation(); onclick?.(e)}} class={cn(
    "flex w-8 h-8 items-center justify-center rounded-lg text-muted-foreground disabled:opacity-50 transition-colors",
    !disabled && "cursor-pointer",
    className
)} {disabled} {...restProps}>
    <span class="w-4 h-4">
        {@render children?.()}
    </span>
</button>