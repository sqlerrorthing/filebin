<script lang="ts">
    import type { HTMLAttributes } from "svelte/elements";
    import type { Snippet } from "svelte";
    import { cn } from "$lib/utils";

    interface Props extends HTMLAttributes<HTMLDivElement> {
        variant: "error";
        onClose?: () => void;
        children?: Snippet;
    }

    let {
        variant,
        onClose,
        class: className,
        children,
        ...restProps
    }: Props = $props();
</script>

<div
        {...restProps}
        class={cn(`
        @container h-full w-full rounded-xl border border-solid p-4 relative`,
        variant === "error" &&
            "bg-destructive/20 border-destructive",
        className
    )}
>
    {@render children?.()}
</div>