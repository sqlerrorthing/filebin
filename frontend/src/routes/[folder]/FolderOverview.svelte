<script lang="ts">
    import Container from "$lib/components/container/Container.svelte";
    import {type EncryptedFolderContext, setEncryptedFolderContext} from "$lib/context/folder.svelte";
    import Bubble from "$lib/components/bubble/Bubble.svelte";
    import FolderHeader from "./FolderHeader.svelte";
    import {onMount} from "svelte";

    let {
        ctx = $bindable()
    }: {
        ctx: EncryptedFolderContext;
    } = $props();

    setEncryptedFolderContext(() => ctx);
</script>

<Container>
    {#if ctx.decrypted.state === "decrypted"}
        <div>
            <FolderHeader />
        </div>
    {:else if ctx.decrypted.state === "error"}
        <Bubble variant="error">
            <span>
                {ctx.decrypted.error}
            </span>
        </Bubble>
    {/if}
</Container>
