import { morph } from '@alpinejs/morph';
import Alpine, { data } from 'alpinejs';
import { sse } from './sse';
import './index.css'
import pad from './pad';

window["Alpine"] = Alpine

Alpine.plugin(morph);
Alpine.plugin(sse);
Alpine.data('pad', pad)

Alpine.start()

