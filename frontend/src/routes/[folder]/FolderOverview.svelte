<script lang="ts">
    import Container from "$lib/components/container/Container.svelte";
    import type { EncryptedFolderContext } from "$lib/context/folder.svelte";

    let ctx: {
        ctx: EncryptedFolderContext;
    } = $props();
</script>

<Container></Container>
