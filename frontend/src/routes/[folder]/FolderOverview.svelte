<script lang="ts">
    import Container from "$lib/components/container/Container.svelte";
    import type { EncryptedFolderContext } from "$lib/context/folder.svelte";
    import Bubble from "$lib/components/bubble/Bubble.svelte";

    let {ctx}: {
        ctx: EncryptedFolderContext;
    } = $props();
</script>

<Container>
    {#if ctx.decrypted.state === "decrypted"}
        <p>
            {ctx.decrypted.ctx.name}
        </p>
    {:else if ctx.decrypted.state === "error"}
        <Bubble variant="error">
            <span>
                {ctx.decrypted.error}
            </span>
        </Bubble>
    {/if}
</Container>
