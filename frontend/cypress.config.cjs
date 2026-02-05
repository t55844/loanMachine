const { defineConfig } = require('cypress');

module.exports = defineConfig({
  e2e: {
    baseUrl: 'http://host.docker.internal:5173',  // Host URL; override with env var in Docker
    viewportWidth: 1280,
    viewportHeight: 720,
    video: true,  // Record videos of tests
    retries: 2,  // Auto-retry flaky tests (good for Web3/async)
    setupNodeEvents(on, config) {
      // Add plugins if needed (e.g., for Web3 wallet simulation)
    },
  },
});