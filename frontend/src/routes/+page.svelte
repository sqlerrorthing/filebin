<script lang="ts">
    import {folderClient, useGrpc} from "$lib/grpc";

    const limits = useGrpc(folderClient.limits);
</script>

<button onclick={() => limits.call({})} disabled={limits.loading}>
    {limits.loading ? 'Loading...' : 'Get limits'}
</button>

{#if limits.error}
    <p style="color: red">{limits.error.message}</p>
{/if}

{#if limits.data}
    <pre>{limits.data.maxFileSize.toString()}</pre>
{/if}
