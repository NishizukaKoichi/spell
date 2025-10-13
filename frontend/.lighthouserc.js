module.exports = {
  ci: {
    collect: {
      url: ['https://magicspell.io'],
      numberOfRuns: 3,
      settings: {
        preset: 'desktop',
      },
    },
    assert: {
      preset: 'lighthouse:recommended',
      assertions: {
        'categories:performance': ['error', {minScore: 0.8}],
        'categories:accessibility': ['error', {minScore: 0.9}],
        'categories:best-practices': ['error', {minScore: 0.9}],
        'categories:seo': ['error', {minScore: 0.9}],
        // Specific assertions
        'first-contentful-paint': ['error', {maxNumericValue: 2000}],
        'interactive': ['error', {maxNumericValue: 3500}],
        'speed-index': ['error', {maxNumericValue: 3000}],
        'largest-contentful-paint': ['error', {maxNumericValue: 2500}],
        'cumulative-layout-shift': ['error', {maxNumericValue: 0.1}],
      },
    },
    upload: {
      target: 'temporary-public-storage',
    },
  },
};
