import { defineConfig, devices } from '@playwright/test';

export default defineConfig({
  testDir: './tests',  
  fullyParallel: false,
  forbidOnly: !!process.env.CI,
  retries: 0,  
  workers: undefined,
  reporter: 'html',  
  use: {
    baseURL: process.env.PLAYWRIGHT_BASE_URL || 'http://host.docker.internal:5173',  
    trace: 'on',  
    video: 'on',  
    viewport: { width: 1280, height: 720 },  
  },
  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
    // Uncomment for cross-browser:
    // { name: 'firefox', use: { ...devices['Desktop Firefox'] } },
    // { name: 'webkit', use: { ...devices['Desktop Safari'] } },
  ],
});