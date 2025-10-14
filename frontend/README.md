# Spell Platform - Caster Portal

Frontend application for the Spell Platform, built with Next.js 14, TypeScript, and Tailwind CSS.

## Tech Stack

- **Framework**: Next.js 14 (App Router)
- **Language**: TypeScript
- **Styling**: Tailwind CSS v4 + shadcn/ui
- **Forms**: React Hook Form + Zod
- **Data Fetching**: SWR
- **Payment**: Stripe Elements (to be integrated)

## Getting Started

### Prerequisites

- Node.js 18.17 or later
- npm or yarn

### Installation

```bash
npm install
```

### Development

```bash
npm run dev
```

Open [http://localhost:3000](http://localhost:3000) with your browser.

### Environment Variables

Create a `.env.local` file:

```env
# Production: Uses relative paths through Cloudflare Worker reverse proxy
# No NEXT_PUBLIC_API_BASE needed - defaults to same origin

# For local backend development:
NEXT_PUBLIC_API_BASE=http://localhost:8080
```

**Production Setup**: The frontend uses relative paths (e.g., `/api/*`, `/auth/*`) which are routed through the Cloudflare Worker reverse proxy to the appropriate backends. No additional configuration is needed for production.

## Project Structure

```
app/
├── login/          # GitHub OAuth login page
├── dashboard/      # Protected dashboard pages
│   ├── layout.tsx  # Dashboard layout with navigation
│   └── page.tsx    # Main dashboard
├── globals.css     # Global styles and theme variables
└── layout.tsx      # Root layout
```

## Features

### Phase 5.1: Frontend Foundation ✅
- Next.js 14 with TypeScript
- Tailwind CSS v4 with shadcn/ui theming
- Login page
- Dashboard layout

### Phase 5.2: Authentication Flow (Upcoming)
- GitHub OAuth integration
- Session management with SWR
- Protected routes middleware

### Phase 5.3: Card Registration (Upcoming)
- Stripe SetupIntent integration
- Card registration form

### Phase 5.4-5.7: Additional Features (Upcoming)
- Budget management UI
- Usage tracking display
- API Key management
- Monthly billing

## Development Notes

This project uses Tailwind CSS v4, which uses `@tailwindcss/postcss` instead of the traditional `tailwind.config.js`. Theme configuration is done through CSS variables in `app/globals.css`.

## Learn More

To learn more about Next.js, take a look at the following resources:

- [Next.js Documentation](https://nextjs.org/docs)
- [Learn Next.js](https://nextjs.org/learn)
- [Tailwind CSS v4](https://tailwindcss.com/docs)
- [shadcn/ui](https://ui.shadcn.com/)
