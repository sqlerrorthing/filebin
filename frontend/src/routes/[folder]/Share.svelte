<script lang="ts">
    import * as m from "$lib/paraglide/messages";
    import Code from "$lib/components/share/Code.svelte";

    let shareState: {
        state: "disconnected"
    } = $state({state: "disconnected"});
</script>

<div class="p-4 bg-card border rounded-xl border-border flex flex-col min-w-58">
    <span class="uppercase text-muted-foreground text-sm">{m["folder.share.share.code"]()}</span>

    <div>
        <Code code={{
            code: null,
            placeholderLen: 6,
        }} onShow={() => {}} onHide={() => {}} />
    </div>
</div>
