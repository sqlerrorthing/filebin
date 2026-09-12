<script lang="ts">
    import * as m from "$lib/paraglide/messages";
    import ActionButton from "./ActionButton.svelte";
</script>

<div
    class="mx-auto flex min-h-screen w-full max-w-7xl p-4 sm:p-6 md:items-center
        lg:p-8"
>
    <div
        class="flex w-full flex-col gap-4 md:flex-row md:items-center
            md:justify-between"
    >
        <div class="flex flex-col gap-4">
            <h2 class="text-muted-foreground text-sm uppercase">
                {m["index.subtitle"]()}
            </h2>

            <h1
                class="text-[clamp(46px,7vw,82px)] leading-[.94] font-bold
                    tracking-[-.065em] whitespace-pre-line"
            >
                {m["index.title"]()}
            </h1>

            <p class="text-md text-muted-foreground max-w-lg font-bold">
                {m["index.description"]()}
            </p>
        </div>

        <div class="flex flex-col gap-2">
            <ActionButton basicKey="index.actions.upload" />
            <ActionButton basicKey="index.actions.join" />
        </div>
    </div>
</div>
