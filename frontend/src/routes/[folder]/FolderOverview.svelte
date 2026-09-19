<script lang="ts">
    import Container from "$lib/components/container/Container.svelte";
    import {type FolderContext, getFolderContext, setFolderContext} from "$lib/context/folder.svelte";
    import Bubble from "$lib/components/bubble/Bubble.svelte";
    import FolderHeader from "./FolderHeader.svelte";
    import {onMount} from "svelte";
    import Share from "./Share.svelte";
    import {FilesContext, setFilesContext} from "$lib/context/files.svelte";
    import FilesTable from "$lib/components/files-table/FilesTable.svelte";

    let {
        ctx = $bindable()
    }: {
        ctx: FolderContext;
    } = $props();

    setFolderContext(() => ctx);
    
    setFilesContext(new FilesContext(
        getFolderContext()
    ));
</script>

<svelte:head>
    {#if ctx.decrypted.state === "decrypted"}
        <title>{ctx.decrypted.ctx.name} — Filebin</title>
    {/if}
</svelte:head>

<Container>
    {#if ctx.decrypted.state === "decrypted"}
        <div class="flex gap-2 flex-col md:flex-row">
            <div class="flex-1">
                <FolderHeader/>
                <div class="mt-2">
                    <FilesTable />
                </div>
            </div>

            <div>
                <Share/>
            </div>
        </div>
    {:else if ctx.decrypted.state === "error"}
        <Bubble variant="error">
            <span>
                {ctx.decrypted.error}
            </span>
        </Bubble>
    {/if}
</Container>
