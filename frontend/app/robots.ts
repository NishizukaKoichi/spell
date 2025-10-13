import { MetadataRoute } from 'next'

export default function robots(): MetadataRoute.Robots {
  // Set to Disallow during development/testing
  // Change to Allow when ready for public launch
  const isProduction = process.env.VERCEL_ENV === 'production'
  const allowCrawling = process.env.NEXT_PUBLIC_ALLOW_CRAWLING === 'true'

  if (!isProduction || !allowCrawling) {
    return {
      rules: {
        userAgent: '*',
        disallow: '/',
      },
    }
  }

  return {
    rules: {
      userAgent: '*',
      allow: '/',
      disallow: ['/api/', '/dashboard/'],
    },
    sitemap: 'https://magicspell.io/sitemap.xml',
  }
}
