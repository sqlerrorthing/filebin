<script lang="ts">
    import Upload from "$lib/components/upload/Upload.svelte";
    import JoinShare from "$lib/components/share/JoinShare.svelte";
    import {limitsStore} from "$lib/stores/limits.svelte";
    import * as m from "$lib/paraglide/messages";
    import {formatBytes} from "$lib/utils";
</script>

<div class="flex justify-center flex-col items-center gap-4 w-full max-w-md mx-auto">
    <Upload/>
    <span class="text-muted-foreground text-sm">
        {m["files.limit"]({limit: limitsStore.loading ? "..." : formatBytes(limitsStore.data?.maxFileSize ?? 0) ?? "Error"})}
    </span>
    <JoinShare/>
</div>