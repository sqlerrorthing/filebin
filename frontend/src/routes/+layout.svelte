<script lang="ts">
    import "./layout.css";
    import { ModeWatcher } from "mode-watcher";
    import Header from "$lib/components/header/Header.svelte";
    import { Toaster } from "svelte-sonner";
    import favicon from "$lib/assets/favicon.ico";

    let { children } = $props();
</script>

<svelte:head>
    <meta property="og:type" content="website" />
    <link rel="icon" href={favicon} />
</svelte:head>

<ModeWatcher />

<Toaster
    position="top-right"
    style="top: 5rem;"
    toastOptions={{
        unstyled: true,
        classes: {
            toast: "group flex w-full items-center gap-3 rounded-xl border border-border bg-popover p-4 text-popover-foreground shadow-lg transition-all",
            content: "flex flex-col gap-0.5 min-w-0 flex-1",

            title: "text-sm font-semibold tracking-tight leading-none text-foreground",
            description: "text-xs text-muted-foreground leading-normal",

            icon: "relative flex items-center justify-center shrink-0 size-5 text-foreground [&_[data-icon]]:flex [&_[data-icon]]:items-center [&_[data-icon]]:justify-center [&_[data-icon]]:size-full [&_svg]:size-5",
            actionButton:
                "inline-flex shrink-0 items-center justify-center rounded-md bg-primary px-3 py-1.5 text-xs font-medium text-primary-foreground shadow transition-colors hover:bg-primary/90 focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring",
            cancelButton:
                "inline-flex shrink-0 items-center justify-center rounded-md bg-secondary px-3 py-1.5 text-xs font-medium text-secondary-foreground transition-colors hover:bg-secondary/80 focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring",
            closeButton:
                "absolute right-2 top-2 rounded-md p-1 text-muted-foreground/50 opacity-0 transition-opacity hover:text-foreground group-hover:opacity-100",

            default: "border-border bg-popover text-popover-foreground",
            success:
                "border-emerald-500/30 bg-emerald-500/10 text-emerald-900 dark:text-emerald-200 [&_[data-icon]]:text-emerald-500",
            error: "border-destructive/30 bg-destructive/10 text-destructive dark:text-red-300 [&_[data-icon]]:text-destructive",
            warning:
                "border-amber-500/30 bg-amber-500/10 text-amber-900 dark:text-amber-200 [&_[data-icon]]:text-amber-500",
            info: "border-sky-500/30 bg-sky-500/10 text-sky-900 dark:text-sky-200 [&_[data-icon]]:text-sky-500",
        },
    }}
/>

<div class="bg-background text-foreground min-h-screen">
    <header class="border-border h-16 border-b">
        <Header />
    </header>

    <main class="flex min-h-[calc(100vh-4rem)] flex-col">
        {@render children()}
    </main>
</div>
