import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

// https://vite.dev/config/
export default defineConfig({
  plugins: [react()],
  server: {
    host: true,                // Listen on all interfaces (0.0.0.0)
    port: 5173,
    strictPort: true,

    // ⭐ Critical fix for Cypress 403: explicitly allow the internal Docker hostname
    allowedHosts: [
      'localhost',
      '127.0.0.1',
      'frontend',              // ← This is what Cypress uses
      '.localhost',            // Optional, covers subdomains
    ],

    hmr: {
      clientPort: 5173,        // Keeps HMR working in local browser (ws://localhost:5173)
      // Do NOT set hmr.host here — it can conflict
    },

    watch: {
      usePolling: true         // Reliable file watching in Docker
    }
  }
});