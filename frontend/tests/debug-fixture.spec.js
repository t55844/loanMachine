// tests/test-fixture.spec.js
import { test, expect } from './fixtures.js';

test('test the setupLoggedInUser fixture directly', async ({ page, setupLoggedInUser }) => {
  // The fixture should automatically run
  console.log('Fixture should have run');
  
  // Check if we're on the main app
  try {
    await expect(page.getByRole('heading', { name: 'Loan Machine DApp' })).toBeVisible({ timeout: 10000 });
    console.log('✓ Main app loaded');
    await page.screenshot({ path: 'test-results/fixture-success.png' });
  } catch (error) {
    console.error('✗ Main app NOT loaded');
    console.log('Current URL:', page.url());
    await page.screenshot({ path: 'test-results/fixture-failure.png', fullPage: true });
  }
});