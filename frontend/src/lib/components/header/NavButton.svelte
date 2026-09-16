<script lang="ts">
    import type { Snippet } from "svelte";
    import type { HTMLButtonAttributes } from "svelte/elements";
    import { cn } from "$lib/utils";

    interface Props extends HTMLButtonAttributes {
        children?: Snippet;
    }

    let { children, class: className, ...restProps }: Props = $props();
</script>

<button
    type="button"
    class={cn(
        `text-muted-foreground hover:bg-accent hover:text-accent-foreground flex
        size-9 items-center justify-center rounded-lg transition-colors`,
        className
    )}
    {...restProps}
>
    {@render children?.()}
</button>
