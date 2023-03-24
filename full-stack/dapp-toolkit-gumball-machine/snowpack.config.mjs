/** @type {import("snowpack").SnowpackUserConfig } */
export default {
    mount: {
      public: { url: '/', static: true },
      src: { url: '/dist' },
    },
    plugins: [
      /* ... */
    ],
    routes: [
      /* Enable an SPA Fallback in development: */
      // {"match": "routes", "src": ".*", "dest": "/index.html"},
    ],
    optimize: {
      /* Example: Bundle your final build: */
      // "bundle": true,
    },
    packageOptions: {
      /* ... */
    },
    devOptions: {
      /* ... */
    },
    buildOptions: {
      /* ... */
    },
    optimize: {
      bundle: true,
      minify: true,
      target: 'es2018',
      entrypoints : ['src/index.js']
    }
  };