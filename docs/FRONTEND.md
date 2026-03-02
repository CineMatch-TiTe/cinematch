# Frontend

## Overview

The frontend is a **Next.js 16** application using the **App Router** with **React 19** and **TypeScript**. It communicates with the Rust backend via a Next.js rewrite proxy.

## Tech Stack

| Technology | Version | Purpose |
|-----------|---------|---------|
| Next.js | 16.1.2 | Framework (App Router, SSR, standalone output) |
| React | 19.2.3 | UI library |
| TypeScript | 5.x | Type safety |
| Tailwind CSS | 4.x | Utility-first styling |
| Radix UI | latest | Accessible headless primitives (Dialog, Tabs, Avatar, etc.) |
| SWR | 2.x | Client-side data fetching with caching |
| Zod | 4.x | Runtime schema validation |
| Orval | 8.x | OpenAPI → TypeScript client code generation |
| Lucide React | latest | Icon library |
| Sonner | 2.x | Toast notifications |
| next-themes | 0.4.x | Dark/light theme support |
| React Compiler | 1.0.0 | Automatic memoization |

## Directory Structure

```
frontend/src/
├── app/                      # Next.js App Router
│   ├── layout.tsx                # Root layout (providers, fonts)
│   ├── page.tsx                  # Home page (join/create party)
│   ├── manifest.tsx              # PWA manifest
│   ├── globals.css               # Tailwind entry + custom styles
│   ├── api/                      # API route handlers
│   ├── create-party/             # Party creation page
│   ├── dashboard/                # User dashboard
│   ├── party-room/               # Main party experience
│   └── preferences/              # User taste preferences
├── components/               # React components
│   ├── ui/                       # shadcn/ui primitives (14 components)
│   ├── party/                    # Party-specific (18 components)
│   ├── preferences/              # Preference forms (5 components)
│   ├── dashboard/                # Dashboard widgets
│   ├── home/                     # Home page components
│   ├── forms/                    # Reusable form components
│   ├── common/                   # Shared components
│   ├── providers/                # Context providers
│   └── user/                     # User-related components
├── hooks/                    # Custom React hooks
│   ├── useMoviePicker.ts         # Movie selection logic
│   ├── useVoting.ts              # Voting state management
│   ├── usePartyViewLogic.ts      # Party page orchestration
│   ├── usePreferences.ts         # Preference CRUD
│   ├── usePhaseCountdown.ts      # Phase timer
│   └── useDeadlineCountdown.ts   # Deadline timer
├── server/                   # Server actions (by domain)
│   ├── auth/                     # Login, logout
│   ├── party/                    # Party CRUD
│   ├── picking/                  # Movie picking
│   ├── voting/                   # Voting actions
│   ├── movie/                    # Movie search/details
│   ├── user/                     # User profile ops
│   ├── recommendation/           # Recommendation fetch
│   ├── leader-tools/             # Leader-only actions
│   ├── member-ops/               # Member operations
│   ├── websocket/                # WS connection management
│   └── system/                   # System info
├── model/                    # Orval-generated API client
├── types/                    # Custom TypeScript types
└── lib/                      # Utilities (cn, etc.)
```

## Pages

| Route | Component | Description |
|-------|-----------|-------------|
| `/` | `page.tsx` | Home — join a party by code or create one |
| `/create-party` | `create-party/page.tsx` | Party creation flow |
| `/dashboard` | `dashboard/page.tsx` | User dashboard with history |
| `/party-room` | `party-room/page.tsx` | Main party experience (lobby, picking, voting, watching, review) |
| `/preferences` | `preferences/page.tsx` | User taste preference editor |

## API Proxy

The Next.js `rewrites` config proxies all `/api/*` requests to the Rust backend:

```ts
// next.config.ts
async rewrites() {
  return [{
    source: '/api/:path*',
    destination: `${process.env.NEXT_PUBLIC_API_BASE}/api/:path*`,
  }]
}
```

This avoids CORS issues and keeps the backend URL hidden from the client.

## Data Fetching Pattern

- **SWR** handles all client-side GET requests with automatic caching and revalidation
- **Server actions** (`src/server/`) wrap `fetch` calls for mutations (POST, PUT, DELETE, PATCH)
- **Orval** generates typed request/response models from the backend's OpenAPI spec

## Styling

- **Tailwind CSS v4** with `@tailwindcss/postcss` plugin
- **tw-animate-css** for animation utilities
- **shadcn/ui** components configured via `components.json`
- **class-variance-authority (CVA)** for component variants
- Theme switching via `next-themes`

## PWA Support

`manifest.tsx` exports a comprehensive web app manifest for installability.
