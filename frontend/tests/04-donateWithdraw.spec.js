import { test, expect } from './fixtures.js';

test.describe('Donation Sidebar', () => {
    test.beforeEach(async ({ page, setupLoggedInUser }) => {
    //console.log('Running setup before test...');
  });


  test('should donate successfully and update balances', async ({ page, extractCurrency, waitForTransaction }) => {
    // Wait for main app to load
    await expect(page.getByRole('heading', { name: /Loan Machine DApp/i })).toBeVisible({ timeout: 30000 });
    await page.waitForTimeout(3000); // Buffer for rendering

     //console.log('Opening sidebar...');
    await page.waitForTimeout(2000); // Wait for UI to stabilize
    
    const menuButton = page.locator('button.menu-toggle.left');
    await menuButton.click({ force: true });
    
    const sidebar = page.locator('.side-menu.left.visible');
    await expect(sidebar).toBeVisible({ timeout: 10000 });
    //console.log('✓ Sidebar opened');
    
    // Wait for sidebar content to load
    await page.waitForTimeout(1500);

    // Click on "Doar" tab
    //console.log('Clicking donate tab...');
    const donateTab = page.locator('[data-cy="donate-tab"]');
    await expect(donateTab).toBeVisible({ timeout: 5000 });
    await donateTab.click({ force: true });
    
    // Wait for tab to activate
    await page.waitForTimeout(1000);

    // Get initial values
    //console.log('Getting initial values...');
    
    // Get initial donations - FIXED SELECTOR
    const donationsLocator = page.locator('.side-menu.left.visible .stat-item:has-text("Doações do Usuário:") span');
    const donationsText = await donationsLocator.innerText({ timeout: 5000 });
    const initialDonations = await extractCurrency(donationsText);
    //console.log(`Initial donations: ${initialDonations}`);

    // Get initial balance - FIXED SELECTOR (matches your HTML)
    const balanceLocator = page.locator('.side-menu.left.visible .balance-amount');
    const balanceText = await balanceLocator.innerText({ timeout: 5000 });
    const initialBalance = await extractCurrency(balanceText);
    //console.log(`Initial balance: ${initialBalance}`);

    // Enter donation amount
    //console.log('Entering donation amount...');
    const donateInput = page.locator('.side-menu.left.visible .donate-input');
    await expect(donateInput).toBeVisible({ timeout: 5000 });

    await donateInput.click({ force: true });
    await donateInput.fill('100', { delay: 100 });  // Slow typing to allow app reactions
    await expect(donateInput).toHaveValue('100');
    
    // Wait a moment
    await page.waitForTimeout(500);

    // Click aprove button
    //console.log('Clicking donate button...');
    const aprovButton = page.locator('.side-menu.left.visible .approve-button');
    await expect(aprovButton).toBeEnabled({ timeout: 5000 });
    await aprovButton.click({ force: true });

    // Click donate button
    //console.log('Clicking donate button...');
    const donateButton = page.locator('.side-menu.left.visible .donate-button');
    await expect(donateButton).toBeEnabled({ timeout: 5000 });
    await donateButton.click({ force: true });

    // Wait for transaction confirmation
    //console.log('Waiting for transaction...');
    await waitForTransaction(45000);

    // Close and reopen sidebar for refresh
    //console.log('Refreshing sidebar...');
    await menuButton.click({ force: true });
    await expect(sidebar).not.toBeVisible({ timeout: 10000 });
    await page.waitForTimeout(2000); // Wait for close animation
    
    await menuButton.click({ force: true });
    await expect(sidebar).toBeVisible({ timeout: 10000 });
    await page.waitForTimeout(1500); // Wait for content to load
    
    // Re-click donate tab to ensure we're in the right section
    await donateTab.click({ force: true });
    await page.waitForTimeout(1000);

    // Get new values
    //console.log('Getting new values...');
    
    const newDonationsText = await donationsLocator.innerText({ timeout: 5000 });
    const newDonations = await extractCurrency(newDonationsText);
    //console.log(`New donations: ${newDonations}`);
    
    await expect(newDonations).toBeCloseTo(initialDonations + 100, 0.01);
    //console.log('✓ Donations increased correctly');

    const newBalanceText = await balanceLocator.innerText({ timeout: 5000 });
    const newBalance = await extractCurrency(newBalanceText);
    //console.log(`New balance: ${newBalance}`);
    
    await expect(newBalance).toBeCloseTo(initialBalance - 100, 0.01);
    //console.log('✓ Balance decreased correctly');

    // Reload page to check transaction list
    //console.log('Reloading page to check transactions...');
    await page.reload();
    await expect(page.getByRole('heading', { name: /Loan Machine DApp/i })).toBeVisible({ timeout: 10000 });
    await page.waitForTimeout(2000);

    // Check transaction - FIXED SELECTORS
    //console.log('Checking transaction history...');
    const transactionRows = page.locator('.transaction-row');
    const rowCount = await transactionRows.count();
    
    if (rowCount > 0) {
        const firstTransaction = transactionRows.first();
        const transactionText = await firstTransaction.textContent();
        //console.log('Transaction text:', transactionText);
        
        // Extract value from transaction text
        if (transactionText && transactionText.includes('Valor:')) {
            const valueMatch = transactionText.match(/Valor:\s*([\d.,]+)/);
            if (valueMatch) {
                const transactionValue = await extractCurrency(valueMatch[1]);
                //console.log(`Transaction value: ${transactionValue}`);
                await expect(transactionValue).toBeCloseTo(100, 0.01);
            }
        }
        
        // Check type
        if (transactionText && transactionText.includes('Doação')) {
            //console.log('✓ Transaction type is Doação');
        }
    } else {
        //console.log('No transactions found yet - might need to wait for indexing');
        await page.waitForTimeout(5000); // Wait more for GraphQL indexing
    }
    
    //console.log('✓ Donation test completed successfully!');
  });

  test('should withdrawal successfully and update balances', async ({ page, extractCurrency, waitForTransaction }) => {
    // Wait for main app to load
    await expect(page.getByRole('heading', { name: /Loan Machine DApp/i })).toBeVisible({ timeout: 30000 });
    await page.waitForTimeout(3000);


    //console.log('Opening sidebar...');
    await page.waitForTimeout(2000); // Wait for UI to stabilize
    
    const menuButton = page.locator('button.menu-toggle.left');
    await menuButton.click({ force: true });
    
    // Now open sidebar again for withdrawal test
    //console.log('Opening sidebar for withdrawal...');
    const sidebar = page.locator('.side-menu.left.visible');
    await expect(sidebar).toBeVisible({ timeout: 10000 });
    //console.log('✓ Sidebar opened');
    
    // Wait for sidebar content to load
    await page.waitForTimeout(1500);

    await expect(sidebar).toBeVisible({ timeout: 10000 });
    await page.waitForTimeout(1500);

    // Click on "Retirar" tab
    //console.log('Clicking withdraw tab...');
    const withdrawTab = page.locator('[data-cy="withdraw-tab"]');
    await expect(withdrawTab).toBeVisible({ timeout: 5000 });
    await withdrawTab.click({ force: true });
    
    await page.waitForTimeout(1000);

    // Get initial values
    //console.log('Getting initial values...');
    
    // Get current balance
    const balanceLocator = page.locator('.side-menu.left.visible .balance-amount');
    const balanceText = await balanceLocator.innerText({ timeout: 5000 });
    const initialBalance = await extractCurrency(balanceText);
    //console.log(`Initial balance: ${initialBalance}`);

    // Get current donations (which should be withdrawable)
    const withdrawLocator = page.locator('.withdrawable-info:has-text("Saldo Disponível para Saque: ") strong');
    const withdrawText = await withdrawLocator.innerText({ timeout: 5000 });
    const initialWithdrawable = await extractCurrency(withdrawText);
    //console.log(`Available to withdraw (donations): ${initialWithdrawable}`);

    // Enter withdrawal amount
    //console.log('Entering withdrawal amount...');
    const withdrawInput = page.locator('.side-menu.left.visible .donate-input.withdraw-input, .side-menu.left.visible .withdraw-input, .side-menu.left.visible input[placeholder*="USD"]');
    await expect(withdrawInput.first()).toBeVisible({ timeout: 5000 });
    
    await withdrawInput.first().click({ force: true });
    await withdrawInput.first().fill('10', { delay: 100 });
    await expect(withdrawInput.first()).toHaveValue('10');
    
    await page.waitForTimeout(500);

    // Click withdraw button
    //console.log('Clicking withdraw button...');
    const withdrawButton = page.locator('.side-menu.left.visible .donate-button.withdraw-button');
    await expect(withdrawButton.first()).toBeEnabled({ timeout: 5000 });
    await withdrawButton.first().click({ force: true });

    // Wait for transaction confirmation
    //console.log('Waiting for transaction...');
    await waitForTransaction(45000);

    // Close and reopen sidebar for refresh
    //console.log('Refreshing sidebar...');
    await menuButton.click({ force: true });
    await expect(sidebar).not.toBeVisible({ timeout: 10000 });
    await page.waitForTimeout(2000);
    
    await menuButton.click({ force: true });
    await expect(sidebar).toBeVisible({ timeout: 10000 });
    await page.waitForTimeout(1500);
    
    // Re-click withdraw tab
    await withdrawTab.click({ force: true });
    await page.waitForTimeout(1000);

    // Get new values
    //console.log('Getting new values...');
    
    const newDonationsText = await withdrawLocator.innerText({ timeout: 5000 });
    const newWithdrawable = await extractCurrency(newDonationsText);
    //console.log(`New donations/withdrawable: ${newWithdrawable}`);
    
    await expect(newWithdrawable).toBeCloseTo(initialWithdrawable - 10, 0.01);
    //console.log('✓ Withdrawable amount decreased correctly');

    const newBalanceText = await balanceLocator.innerText({ timeout: 5000 });
    const newBalance = await extractCurrency(newBalanceText);
    //console.log(`New balance: ${newBalance}`);
    
    await expect(newBalance).toBeCloseTo(initialBalance + 10, 0.01);
    //console.log('✓ Balance increased correctly');

    // Reload page to check transaction list
    //console.log('Reloading page to check transactions...');
    await page.reload();
    await expect(page.getByRole('heading', { name: /Loan Machine DApp/i })).toBeVisible({ timeout: 10000 });
    await page.waitForTimeout(2000);

    // Check transaction
    //console.log('Checking transaction history...');
    const transactionRows = page.locator('.transaction-row');
    const rowCount = await transactionRows.count();
    
    if (rowCount > 0) {
        // Look for the most recent transaction (should be withdrawal)
        for (let i = 0; i < Math.min(rowCount, 5); i++) {
            const transaction = transactionRows.nth(i);
            const transactionText = await transaction.textContent();
            
            if (transactionText && transactionText.includes('Retirada')) {
                //console.log('Found withdrawal transaction');
                
                // Extract value
                if (transactionText.includes('Valor:')) {
                    const valueMatch = transactionText.match(/Valor:\s*([\d.,]+)/);
                    if (valueMatch) {
                        const transactionValue = await extractCurrency(valueMatch[1]);
                        //console.log(`Withdrawal value: ${transactionValue}`);
                        await expect(transactionValue).toBeCloseTo(10, 0.01);
                        //console.log('✓ Withdrawal recorded correctly');
                        break;
                    }
                }
            }
        }
    } else {
        //console.log('No transactions found yet');
        await page.waitForTimeout(5000);
    }
    
    //console.log('✓ Withdrawal test completed successfully!');
  });
});