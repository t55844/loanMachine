import { test, expect } from './fixtures.js';

test.describe('Wallet Vinculation', () => {
  
  test.beforeEach(async ({ page, setupLoggedInUser }) => {
    //console.log('Running setup before test...');
  });

  test('should link member ID successfully', async ({ page, waitForTransaction }) => {
    //console.log('Test starting, checking if page is loaded...');
    
    // Verify the page is actually loaded
    await expect(page.getByRole('heading', { name: 'Loan Machine DApp' })).toBeVisible({ timeout: 15000 });
    //console.log('Main app loaded');
    
    // Wait a moment for stability
    await page.waitForTimeout(2000);
    
    // Now click the menu button
    //console.log('Clicking menu button...');
    await page.locator('button.menu-toggle.left').click({ timeout: 10000 });

    // Wait for sidebar to open
    await expect(page.locator('.side-menu.left.visible')).toBeVisible({ timeout: 10000 });
    
    // Fill Member ID
    await page
      .locator('.side-menu.left.visible [data-cy="member-id-input"]')
      .fill('321654987');
    
    // Click Vincular
    await page
      .locator('button.vinculate-button:has-text("Vincular Membro")')
      .scrollIntoViewIfNeeded();
      
    await page
      .locator('button.vinculate-button:has-text("Vincular Membro")')
      .click({ force: true, timeout: 10000 });

    
    // Confirm Transaction Modal
    await page
      .locator('.confirmation-modal-overlay')
      .getByRole('button', { name: 'Confirmar Transação' })
      .click();
    
    await expect(page.locator('.confirmation-modal-overlay')).not.toBeVisible({ timeout: 20000 });
    
    // Wait for blockchain confirmation
    await waitForTransaction(45000);
    
    // Click "Tentar Novamente" banner
    await page
      .locator('.wallet-verification-banner .retry-button-banner')
      .click({ force: true });
    
    await expect(page.locator('.wallet-verification-banner')).not.toBeVisible({ timeout: 25000 });
    

    //console.log('Verifying member ID in sidebar...');

// Wait EXTRA long to ensure everything is ready
//console.log('Waiting 5 seconds for UI to stabilize...');
await page.waitForTimeout(5000);

const menuButton = page.locator('button.menu-toggle.left');
const sidebar = page.locator('.side-menu.left.visible');

// Double-click approach to ensure sidebar opens
//console.log('Double-clicking menu button to ensure sidebar opens...');
await menuButton.click({ force: true });
await page.waitForTimeout(1000); // Wait between clicks

// Wait for sidebar
await expect(sidebar).toBeVisible({ timeout: 15000 });
//console.log('✓ Sidebar opened');

// Wait for content
await page.waitForTimeout(2000);

// Simple check - does sidebar contain our member ID?
const hasMemberId = await sidebar.locator(':has-text("321654987")').count() > 0;
if (hasMemberId) {
    //console.log('✓ Member ID found in sidebar!');
} else {
    //console.error('Member ID not found');
    await page.screenshot({ path: 'test-results/error-no-member-id.png' });
    throw new Error('Member ID not displayed after vinculation');
}
  });
});