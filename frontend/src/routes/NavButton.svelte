<script lang="ts">
    import type {Snippet} from "svelte";
    import type {HTMLButtonAttributes} from "svelte/elements";
    import {cn} from "$lib/utils";

    interface Props extends HTMLButtonAttributes {
        children?: Snippet;
    }

    let {
        children,
        class: className,
        ...restProps
    }: Props = $props();
</script>

<button
        type="button"
        class={cn(
            "flex size-9 items-center justify-center rounded-lg text-muted-foreground transition-colors hover:bg-accent hover:text-accent-foreground",
            className
        )}
        {...restProps}
>
    {@render children?.()}
</button>