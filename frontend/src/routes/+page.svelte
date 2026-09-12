<script lang="ts">
    import * as m from "$lib/paraglide/messages";
    import MoveUpRight from "@lucide/svelte/icons/move-up-right";
    import ActionButton from "./ActionButton.svelte";
    import MoveRight from "@lucide/svelte/icons/move-right";
</script>

<div class="mx-auto w-full max-w-7xl p-4 sm:p-6 md:my-auto lg:p-8">
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

        <div class="flex flex-col gap-2 rounded-2xl p-2 shadow-xl">
            <ActionButton
                name={m["index.actions.upload.name"]}
                description={m["index.actions.upload.description"]}
                highlight={true}
                icon={MoveUpRight}
            />

            <ActionButton
                name={m["index.actions.join.name"]}
                description={m["index.actions.join.description"]}
                icon={MoveRight}
            />
        </div>
    </div>
</div>
