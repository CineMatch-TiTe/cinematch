import { defineConfig } from 'orval';

export default defineConfig({
  partyApi: {
    output: {
      mode: 'tags-split',
      target: 'src/party-api.ts',
      schemas: 'src/model',
      client: 'swr',
      mock: true,
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