import { test, expect } from './fixtures.js';


test.describe('Loan Activation and Activity', () => {
    test.beforeEach(async ({ page, setupLoggedInUser }) => {
    //console.log('Running setup before test...');
  });

    test('check if the loan requisition is available to coverage and activation', async ({ page, waitForTransaction }) => {
        // Wait for main app to load
        await expect(page.getByRole('heading', { name: /Sistema de Requisição de Empréstimos/i })).toBeVisible({ timeout: 30000 });
        await page.waitForTimeout(3000); // Buffer for rendering


        await page.locator('[pw-test-id="pending-requisitions-button-tab"]').click({ force: true });
        
        await expect(page.getByRole('heading', { name: /Requisições de Empréstimo Disponíveis/i })).toBeVisible({ timeout: 10000 });

        await expect(page.locator('div.stats-box div div:has-text("Doações Totais:100.00 USD")')).toBeVisible({ timeout: 10000 });
        await expect(page.locator('div.stats-box div div:has-text("Em Cobertura:0.00 USD")')).toBeVisible({ timeout: 10000 });
        await expect(page.locator('div.stats-box div div:has-text("Disponível:90.00 USD")')).toBeVisible({ timeout: 10000 });

        await expect(page.getByRole('heading', { name: /Requisição #1/i })).toBeVisible({ timeout: 10000 });

        await expect(page.locator('.requsitionBlock div div span:has-text("Pendente")').first()).toBeVisible({ timeout: 10000 });

        const valorLocator = page.locator('.requsitionBlock .requisition-item .requisition-details strong:has-text("Valor:")').first();
        await expect(valorLocator).toBeVisible({ timeout: 10000 });
        await expect(valorLocator.locator('..')).toHaveText('Valor: 50.00 USD', { timeout: 10000 });

        const coverageLocator = page.locator('.requsitionBlock .requisition-item .requisition-details strong:has-text("Cobertura:")').first();
        await expect(coverageLocator).toBeVisible({ timeout: 10000 });
        await expect(coverageLocator.locator('..')).toHaveText('Cobertura: 0% / 100%', { timeout: 10000 });

        const creditorsLocator = page.locator('.requsitionBlock .requisition-item .requisition-details strong:has-text("Credores:")').first();
        await expect(creditorsLocator).toBeVisible({ timeout: 10000 });
        await expect(creditorsLocator.locator('..')).toHaveText('Credores: 0', { timeout: 10000 });

        const currentDate = new Date().toLocaleDateString('pt-BR', {timeZone: 'America/Sao_Paulo'});
        const creationTimeLocator = page.locator('.requsitionBlock .requisition-item .requisition-details strong:has-text("Criada em:")').first();
        await expect(creationTimeLocator).toBeVisible({ timeout: 10000 });
        await expect(creationTimeLocator.locator('..')).toHaveText(`Criada em: ${currentDate}`, { timeout: 10000 });

        const mutuatarioLocator = page.locator('.requsitionBlock .requisition-item .requisition-details strong:has-text("Mutuatário:")').first();
        await expect(mutuatarioLocator).toBeVisible({ timeout: 10000 });
        await expect(mutuatarioLocator.locator('..')).toHaveText('Mutuatário: 0xBcd4...4096', { timeout: 10000 });

    });

    test('should update coverage and creditors when covering with 50% via quick selection and custom percentage', async ({ page, waitForTransaction }) => {
    // Wait for main app to load
    await expect(page.getByRole('heading', { name: /Sistema de Requisição de Empréstimos/i })).toBeVisible({ timeout: 30000 });
    await page.waitForTimeout(3000); // Buffer for rendering

    // Navigate to pending requisitions
    await page.locator('[pw-test-id="pending-requisitions-button-tab"]').click({ force: true });
    await expect(page.getByRole('heading', { name: /Requisições de Empréstimo Disponíveis/i })).toBeVisible({ timeout: 10000 });

    await page.locator('.requisition-item').first().click({ force: true });

    // Locate the first requisition block
    const requisitionBlock = page.locator('.requsitionBlock .requisition-item').first();
    //await expect(page.getByRole('heading', { name: /Requisição #1/i })).toBeVisible({ timeout: 10000 });

    // Verify initial requisition details
    await expect(requisitionBlock.locator('span:has-text("Pendente")').first()).toBeVisible();

    const valorLocator = requisitionBlock.locator('strong:has-text("Valor:")').first();
    await expect(valorLocator).toBeVisible();
    await expect(valorLocator.locator('..')).toHaveText('Valor: 50.00 USD');

    const coverageLocator = requisitionBlock.locator('strong:has-text("Cobertura:")').first();
    await expect(coverageLocator).toBeVisible();
    await expect(coverageLocator.locator('..')).toHaveText('Cobertura: 0% / 100%');

    const creditorsLocator = requisitionBlock.locator('strong:has-text("Credores:")').first();
    await expect(creditorsLocator).toBeVisible();
    await expect(creditorsLocator.locator('..')).toHaveText('Credores: 0');

    // Locate the cover section inside the requisition
    const coverSection = requisitionBlock.locator('.cover-loan-section');

    // Verify all headers and labels in the cover section
    await expect(coverSection.locator('h4:has-text("Cobrir Este Empréstimo")')).toBeVisible();
    await expect(coverSection.locator('p:has-text("Seleção rápida:")')).toBeVisible();
    await expect(coverSection.locator('p:has-text("Ou insira a porcentagem personalizada (10-100):")')).toBeVisible();

    // --- Test Quick Selection 50% ---
    const quick50Button = coverSection.locator('.percentage-grid button:has-text("50%")');
    await quick50Button.click();

    // Wait for transaction to complete and UI to update
    // Assuming waitForTransaction fixture waits for the blockchain tx and subsequent re-render
    await waitForTransaction();
    
    const aproveButton = coverSection.getByRole('button', { name: 'Aprovar' });

    await aproveButton.click();
    await waitForTransaction();
    await quick50Button.click();
    await waitForTransaction();
 // Confirm Transaction Modal
    const confirmModalButton = page.locator('.confirmation-modal-overlay').getByRole('button', { name: 'Confirmar Transação' })
      
    await confirmModalButton.click();
    await expect(page.locator('.confirmation-modal-overlay')).not.toBeVisible({ timeout: 10000 });
    
    // Wait for blockchain confirmation
    await waitForTransaction(45000);

    // Verify coverage and creditors updated
    await expect(coverageLocator.locator('..')).toHaveText('Cobertura: 50% / 100%', { timeout: 30000 });
    //await expect(creditorsLocator.locator('..')).toHaveText('Credores: 1', { timeout: 30000 });
    
    await page.locator('.requisition-item').first().click({ force: true });

    // --- Test Custom Percentage 50% ---
    // Fill the custom input with 50
    await coverSection.locator('input.custom-percentage-input').fill('40');
    // Click the "Cobrir" button
    await coverSection.locator('button.custom-cover-button:has-text("Cobrir")').click();
    await waitForTransaction();

    await confirmModalButton.click();
    await expect(page.locator('.confirmation-modal-overlay')).not.toBeVisible({ timeout: 10000 });
    
    // Wait for blockchain confirmation
    await waitForTransaction(45000);

    // Verify final coverage and creditors
    await expect(coverageLocator.locator('..')).toHaveText('Cobertura: 90% / 100%', { timeout: 30000 });
    //await expect(creditorsLocator.locator('..')).toHaveText('Credores: 1', { timeout: 30000 });
    });

})