import {paraglideVitePlugin} from '@inlang/paraglide-js'
import tailwindcss from '@tailwindcss/vite';
import adapter from '@sveltejs/adapter-auto';
import {sveltekit} from '@sveltejs/kit/vite';
import {defineConfig} from 'vite';
import utwm from 'unplugin-tailwindcss-mangle/vite'

export default defineConfig({
    plugins: [
        paraglideVitePlugin({
            project: './project.inlang',
            outdir: './src/lib/paraglide',
            emitTsDeclarations: true
        }),
        tailwindcss(),
        sveltekit({
            compilerOptions: {
                runes: ({filename}) => filename.split(/[/\\]/).includes('node_modules') ? undefined : true
            },

            adapter: adapter()
        }),
        utwm()
    ]
});
