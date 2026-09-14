<script lang="ts">

    import type {HTMLAttributes} from "svelte/elements";
    import type {Snippet} from "svelte";
    import {cn} from "$lib/utils";

    interface Props extends HTMLAttributes<HTMLDivElement> {
        children?: Snippet;
    }

    let {
        children,
        class: className,
    }: Props = $props();
</script>

<div class={cn(
    "mx-auto w-full max-w-7xl p-4 sm:p-6 lg:p-8",
    className
)}>
    {@render children?.()}
</div>