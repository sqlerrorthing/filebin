<script lang="ts">
    import {Eye, EyeOff, LoaderCircle} from "@lucide/svelte";
    import {cn} from "$lib/utils";
    import {toast} from "svelte-sonner";
    import * as m from "$lib/paraglide/messages";

    let {
        code = $bindable(),
        onShow,
        onHide
    }: {
        code:
            | {
            code: string;
            rotation: Date;
        }
            | {
            code: null;
            placeholderLen: number;
            rotation?: never;
        },
        onShow: () => void | Promise<void>,
        onHide: () => void | Promise<void>
    } = $props();

    let loading = $state(false);
    let codeHighlight = $state(false);
    let timeLeft = $state(0);

    let Icon = $derived.by(() => {
        if (loading) {
            return LoaderCircle
        } else if (code === null) {
            return Eye
        } else {
            return EyeOff
        }
    });

    let displayCode = $derived(code.code ?? "•".repeat(code.placeholderLen));
    let isCode = $derived(code.code !== null);

    $effect(() => {
        if (!code.rotation) {
            timeLeft = 0;
            return;
        }

        const update = () => {
            timeLeft = Math.max(0, code.rotation!.getTime() - Date.now());
        };

        update();

        const interval = setInterval(update, 1000);

        return () => clearInterval(interval);
    });

    const formattedTime = $derived.by(() => {
        const totalSeconds = Math.floor(timeLeft / 1000);

        const minutes = Math.floor(totalSeconds / 60);
        const seconds = totalSeconds % 60;

        return `${String(minutes).padStart(2, '0')}:${String(seconds).padStart(2, '0')}`;
    });

    const onclick = async () => {
        try {
            loading = true;

            if (code.code !== null) {
                onHide()
            } else {
                onShow()
            }
        } finally {
            loading = false;
        }
    }

    const copyCode = async () => {
        if (code.code !== null) {
            codeHighlight = true;

            await navigator.clipboard.writeText(code.code);
            toast.success(m["common.actions.copied"](), {duration: 500});

            setTimeout(() => {
                codeHighlight = false;
            }, 100)
        }
    }
</script>

<div class="py-2 flex flex-col">
    <div>
        <button class={cn(
            "inline-flex items-center gap-[0.14em]",
            isCode ? "cursor-pointer" : "text-muted-foreground",
            codeHighlight ? "bg-accent-foreground/30" : (isCode && "hover:bg-accent-foreground/5")
        )}
                onclick={copyCode}
                disabled={loading || !isCode}
        >
            {#each displayCode as char}
                <span
                        class={cn(
                        "inline-flex h-[1.2em] w-[0.7em] items-center justify-center font-mono text-4xl leading-none",
                        isCode && "font-bold"
                    )}
                >
                    {char}
                </span>
            {/each}
        </button>

        <button class={cn(
            "ml-2",
            !loading && "cursor-pointer")
        } disabled={loading} {onclick}>
            <Icon class={cn(loading && "animate-spin")}/>
        </button>
    </div>

    <span class="h-4 text-muted-foreground text-sm leading-none">
        {#if isCode}
            {formattedTime}
        {/if}
    </span>
</div>