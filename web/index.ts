import { morph } from '@alpinejs/morph';
import Alpine, { data } from 'alpinejs';
import { sse } from './sse';
window["Alpine"] = Alpine

Alpine.plugin(morph);
Alpine.plugin(sse);

Alpine.start()

