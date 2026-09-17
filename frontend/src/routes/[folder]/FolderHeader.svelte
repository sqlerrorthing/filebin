<script lang="ts">
    import {getEncryptedFolderContext} from "$lib/context/folder.svelte";

    const ctx = getEncryptedFolderContext();

    const decrypted = $derived.by(() => {
        return ctx().decrypted;
    })
</script>

{#if decrypted.state === "decrypted"}
    {decrypted.ctx.name}
{/if}
