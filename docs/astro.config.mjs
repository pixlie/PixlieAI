// @ts-check
import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';

// https://astro.build/config
export default defineConfig({
	integrations: [
	  starlight({
		title: 'PixlieAI Docs',
		sidebar: [
		  {
			label: 'Getting Started',
			items: [
			  { label: 'Installation', link: '/start/installation' }  // Updated to match new folder name
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
