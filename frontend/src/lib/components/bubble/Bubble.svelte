<script lang="ts">
    import type { HTMLAttributes } from "svelte/elements";
    import type { Snippet } from "svelte";
    import { cn } from "$lib/utils";
    import { X } from "@lucide/svelte";

    interface Props extends HTMLAttributes<HTMLDivElement> {
        variant: "error";
        /// On close doesnt close the error actually
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
    class={cn(
        ` @container relative flex h-full w-full items-start rounded-xl border
        border-solid p-4`,
        variant === "error" && "bg-destructive/20 border-destructive",
        className
    )}
>
    {@render children?.()}

    {#if onClose}
        <button class="ml-auto shrink-0 cursor-pointer">
            <X />
        </button>
    {/if}
</div>
