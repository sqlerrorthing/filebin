<script lang="ts">
    import type {HTMLAttributes} from "svelte/elements";
    import type {Snippet} from "svelte";
    import {cn} from "$lib/utils";

    interface Props extends HTMLAttributes<HTMLDivElement> {
        variant: "error";
        onClose?: () => void;
        children?: Snippet
    }

    let {
        variant,
        onClose,
        class: className,
        children,
        ...restProps
    }: Props = $props();
</script>

<div {...restProps} class={cn(
    "w-full h-full p-4 rounded-xl border-solid border",
    variant === "error" && "bg-destructive/50 border-destructive text-destructive-foreground",
    className
)}>
    {@render children?.()}
</div>