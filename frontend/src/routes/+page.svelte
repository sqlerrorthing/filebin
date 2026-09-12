<script lang="ts">
    import Upload from '$lib/components/upload/Upload.svelte';
    import JoinShare from '$lib/components/share/JoinShare.svelte';
    import { limitsStore } from '$lib/stores/limits.svelte';
    import * as m from '$lib/paraglide/messages';
    import { formatBytes } from '$lib/utils';
</script>

<div
    class="mx-auto flex w-full max-w-md flex-col items-center justify-center gap-4"
>
    <Upload />
    <span class="text-muted-foreground text-sm">
        {m['files.limit-info']({
            limit: limitsStore.loading
                ? '...'
                : (formatBytes(limitsStore.data?.maxFileSize ?? 0) ?? 'Error'),
        })}
    </span>
    <JoinShare />
</div>
