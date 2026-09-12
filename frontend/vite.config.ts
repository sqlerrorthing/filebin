import { paraglideVitePlugin } from '@inlang/paraglide-js';
import tailwindcss from '@tailwindcss/vite';
import adapter from '@sveltejs/adapter-node';
import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';
import Icons from 'unplugin-icons/vite';

export default defineConfig({
    plugins: [
        paraglideVitePlugin({
            project: './project.inlang',
            outdir: './src/lib/paraglide',
            emitTsDeclarations: true,
            strategy: ['url', 'baseLocale'],
        }),
        tailwindcss(),
        sveltekit({
            compilerOptions: {
                runes: ({ filename }) =>
                    filename.split(/[/\\]/).includes('node_modules')
                        ? undefined
                        : true,
            },

            adapter: adapter(),
        }),
        Icons({
            compiler: "svelte",
            autoInstall: true
        })
    ],
    ssr: {
        noExternal: ['@lucide/svelte', '@inlang/paraglide-js-svelte'],
        external: ['@noble/ciphers', '@noble/hashes'],
    },
    optimizeDeps: {
        exclude: ['@noble/ciphers', '@noble/hashes'],
    },
    build: {
        target: 'esnext',
        minify: 'terser',
        terserOptions: {
            compress: {
                passes: 3,
                drop_console: true,
                drop_debugger: true,
                toplevel: true,
                unsafe: true,
                unsafe_arrows: true,
                unsafe_methods: true,
                unsafe_proto: true
            },
            mangle: {
                toplevel: true,
            },

            format: {
                comments: false,
            },
        },

        cssMinify: 'lightningcss',

        sourcemap: false,

        emptyOutDir: true,
    },
});
