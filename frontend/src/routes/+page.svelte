<script lang="ts">
    import Upload from "./Upload.svelte";
    import {limitsStore} from "$lib/stores/limits.svelte";
    import * as m from "$lib/paraglide/messages";
    import {formatBytes} from "$lib/utils";
</script>

<div class="flex justify-center flex-col items-center gap-2">
    <Upload/>
    <span class="text-muted-foreground">
        {m["files.limit"]({limit: limitsStore.loading ? "..." : formatBytes(limitsStore.data?.maxFileSize ?? 0) ?? "Error"})}
    </span>
</div>