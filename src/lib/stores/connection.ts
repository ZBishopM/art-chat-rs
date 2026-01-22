import { writable } from 'svelte/store';
import type { ConnectionState } from '$lib/types';

export const connectionState = writable<ConnectionState>('connecting');
