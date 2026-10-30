import { feed } from '../../lib/feed';
import { translatedPagePaths } from '../../lib/localized-routes.mjs';
import type { APIContext } from 'astro';
export const getStaticPaths = () => translatedPagePaths();
export const GET = (context: APIContext) => feed(context, context.props.locale);
