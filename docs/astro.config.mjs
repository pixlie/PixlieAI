// @ts-check
import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';

// https://astro.build/config
export default defineConfig({
	integrations: [
		starlight({
			title: 'PixlieAI Docs',
			social: {
				github: 'https://github.com/pixlie/PixlieAI',
			},
			sidebar: [
				{
					label: 'Getting Started',
					items: [
						// Each item here is one entry in the navigation menu.
						{ label: 'Getting Started', slug: 'guides/example' },
					],
				},
				{
					label: 'Integrations',
					autogenerate: { directory: 'integrations' },
				},
			],
		}),
	],
});
