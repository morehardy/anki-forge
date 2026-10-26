import { feed } from '../lib/feed';
import type { APIContext } from 'astro';
export const GET = (context: APIContext) => feed(context, 'en');
