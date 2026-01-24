import { defineConfig } from 'orval';

export default defineConfig({

  serverApi: {
    output: {
      mode: 'tags-split',
      target: 'src/server/api.ts',
      schemas: 'src/model',
      client: 'fetch',
      mock: false,
    },
    input: {
      target: './openapi.json',
      override: {
        mutator: {
          path: 'src/lib/orval-client.ts',
          name: 'customInstance',
        },
      },
    },
  },
});