import { paraglideMiddleware } from '$lib/paraglide/server';
import type {Handle, Transport} from '@sveltejs/kit';
import {getTextDirection} from "$lib/paraglide/runtime";

export const handle: Handle = ({ event, resolve }) => {
    return paraglideMiddleware(event.request, ({ request: localizedRequest, locale }) => {
        event.request = localizedRequest;

        return resolve(event, {
            transformPageChunk: ({ html }) => {
                return html
                    .replace('%lang%', locale)
                    .replace('%dir%', getTextDirection(locale));
            }
        });
    });
};
