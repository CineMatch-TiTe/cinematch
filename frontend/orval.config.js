import { defineConfig } from 'orval';

export default defineConfig({

  serverApi: {
    output: {
      mode: 'tags-split',
      target: 'src/server/api.ts',
      schemas: 'src/model',
      client: 'fetch',
      mock: false,
      override: {
        mutator: {
          path: './src/lib/orval-client.ts',
          name: 'customInstance',
        },
      },
    },
    input: {
      target: 'http://localhost:8085/api-docs/openapi.json',
    },
  },
});