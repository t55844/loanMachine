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
    await expect(page.getByRole('heading', { name: /Sistema de Requisição de Empréstimos/i })).toBeVisible({ timeout: 30000 });
    await page.waitForTimeout(3000); 

    await page.locator('[pw-test-id="pending-requisitions-button-tab"]').click({ force: true });
    await expect(page.getByRole('heading', { name: /Requisições de Empréstimo Disponíveis/i })).toBeVisible({ timeout: 10000 });

    await page.locator('.requisition-item').first().click({ force: true });

    const requisitionBlock = page.locator('.requsitionBlock .requisition-item').first();
    await expect(page.getByRole('heading', { name: /Requisição #1/i })).toBeVisible({ timeout: 10000 });

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

    const coverSection = requisitionBlock.locator('.cover-loan-section');

    await expect(coverSection.locator('h4:has-text("Cobrir Este Empréstimo")')).toBeVisible();
    await expect(coverSection.locator('p:has-text("Seleção rápida:")')).toBeVisible();
    await expect(coverSection.locator('p:has-text("Ou insira a porcentagem personalizada (10-100):")')).toBeVisible();

    const quick50Button = coverSection.locator('.percentage-grid button:has-text("50%")');
    await quick50Button.click();

    await waitForTransaction();
    
    const aproveButton = coverSection.getByRole('button', { name: 'Aprovar' });

    await aproveButton.click();
    await waitForTransaction();
    await quick50Button.click();
    await waitForTransaction();
    const confirmModalButton = page.locator('.confirmation-modal-overlay').getByRole('button', { name: 'Confirmar Transação' })
      
    await confirmModalButton.click();
    await expect(page.locator('.confirmation-modal-overlay')).not.toBeVisible({ timeout: 10000 });
    
    await waitForTransaction(45000);
    await expect(page.locator('[pw-test-id="toast-notification"]:has-text("Cobertura de 50% do empréstimo ")')).toBeVisible({ timeout: 30000 });

    await expect(coverageLocator.locator('..')).toHaveText('Cobertura: 50% / 100%', { timeout: 30000 });
    
    await page.locator('.requisition-item').first().click({ force: true });


    await coverSection.locator('input.custom-percentage-input').fill('50');
    await coverSection.locator('button.custom-cover-button:has-text("Cobrir")').click();
    await waitForTransaction();

    await confirmModalButton.click();
    await expect(page.locator('.confirmation-modal-overlay')).not.toBeVisible({ timeout: 10000 });
    
    await waitForTransaction(45000);

    await expect(page.locator('[pw-test-id="toast-notification"]:has-text("Cobertura de 50% do empréstimo ")')).toBeVisible({ timeout: 30000 });
    await page.reload();

    const globalStatusAlterationRow = page.locator('[pw-test-id="transaction-row-0-Empréstimo"]')
    await expect(globalStatusAlterationRow).toBeVisible({ timeout: 30000 });
    await expect(globalStatusAlterationRow.locator('div:has-text("Valor: 50.00 USD")')).toBeVisible();
    await expect(globalStatusAlterationRow.locator('div:has-text("Tipo: Empréstimo")')).toBeVisible();


    const now = new Date();
    const dd = String(now.getDate()).padStart(2, '0');
    const mm = String(now.getMonth() + 1).padStart(2, '0');
    const yyyy = now.getFullYear();
    const currentDate = `${dd}/${mm}/${yyyy}`;
    await expect(globalStatusAlterationRow.locator(`div:has-text("Horário: ${currentDate}")`)).toBeVisible();
    await expect(globalStatusAlterationRow.locator('div:has-text("Carteira: 0xbcd4...4096")')).toBeVisible();

    });

    test('should pay one installment and update loan values correctly', async ({ page, waitForTransaction }) => {
        await expect(page.getByRole('heading', { name: /Sistema de Requisição de Empréstimos/i })).toBeVisible({ timeout: 30000 });
        await page.waitForTimeout(3000); 

        await page.locator('[pw-test-id="user-contracts-button-tab"]').click({ force: true });

        await expect(page.getByRole('heading', { name: /Meus Contratos de Empréstimo/i })).toBeVisible({ timeout: 10000 });

        const firstLoan = page.locator('.requisition-item').first();
        await expect(firstLoan).toBeVisible({ timeout: 10000 });

        const initialDebtText = await firstLoan.locator('.requisition-details div:has-text("Dívida Restante:")').textContent();
        const initialNextPaymentText = await firstLoan.locator('.requisition-details div:has-text("Próximo Pagamento:")').textContent();
        const initialProgressText = await firstLoan.locator('.requisition-details div:has-text("Progresso:")').textContent();

        const initialDebt = parseFloat(initialDebtText.match(/[\d.]+/)[0]);
        const initialNextPayment = parseFloat(initialNextPaymentText.match(/[\d.]+/)[0]);
        const progressMatch = initialProgressText.match(/(\d+)\/(\d+)/);
        const initialPaid = parseInt(progressMatch[1]);
        const totalParcels = parseInt(progressMatch[2]);

        await firstLoan.click({ force: true });

        const expandedSection = firstLoan.locator('.cover-loan-section');
        await expect(expandedSection).toBeVisible({ timeout: 5000 });

        // Handle approval if needed (check for approve button inside expanded section)
        const approveButton = expandedSection.locator('button.approve-button:has-text("Aprovar USD")');
        if (await approveButton.isVisible()) {
            await approveButton.click();
            await waitForTransaction(); // wait for approval transaction

            // After approval, the approve button should disappear and pay button become enabled
            // Optionally wait for the pay button to be enabled
        }

        const payButton = expandedSection.locator('button.repay-button:has-text("Pagar Parcela")');
        await expect(payButton).toBeEnabled({ timeout: 5000 });
        await payButton.click();

        const confirmModalButton = page.locator('.confirmation-modal-overlay').getByRole('button', { name: 'Confirmar Transação' });
        await confirmModalButton.click();
        await expect(page.locator('.confirmation-modal-overlay')).not.toBeVisible({ timeout: 10000 });

        await waitForTransaction(45000);

        const updatedFirstLoan = page.locator('.requisition-item').first();
        await expect(updatedFirstLoan).toBeVisible();

        const updatedDebtText = await updatedFirstLoan.locator('.requisition-details div:has-text("Dívida Restante:")').textContent();
        const updatedNextPaymentText = await updatedFirstLoan.locator('.requisition-details div:has-text("Próximo Pagamento:")').textContent();
        const updatedProgressText = await updatedFirstLoan.locator('.requisition-details div:has-text("Progresso:")').textContent();
        await expect(page.locator('[pw-test-id="toast-notification"]:has-text("Pagamento bem-sucedido para empréstimo")')).toBeVisible({ timeout: 30000 });

        const updatedDebt = parseFloat(updatedDebtText.match(/[\d.]+/)[0]);
        const updatedNextPayment = parseFloat(updatedNextPaymentText.match(/[\d.]+/)[0]);
        const updatedProgressMatch = updatedProgressText.match(/(\d+)\/(\d+)/);
        const updatedPaid = parseInt(updatedProgressMatch[1]);

        expect(updatedDebt).toBeCloseTo(initialDebt - initialNextPayment, 2);
        expect(updatedPaid).toBe(initialPaid + 1);
        expect(updatedNextPayment).toBe(initialNextPayment);    
    
        await waitForTransaction(45000);

        await page.reload();

        const globalStatusAlterationRow = page.locator('[pw-test-id="transaction-row-0-Quitação"]')
        await expect(globalStatusAlterationRow).toBeVisible({ timeout: 30000 });
        await expect(globalStatusAlterationRow.locator('div:has-text("Valor: 4.17 USD")')).toBeVisible();
        await expect(globalStatusAlterationRow.locator('div:has-text("Tipo: Quitação")')).toBeVisible();

        const now = new Date();
        const dd = String(now.getDate()).padStart(2, '0');
        const mm = String(now.getMonth() + 1).padStart(2, '0');
        const yyyy = now.getFullYear();
        const currentDate = `${dd}/${mm}/${yyyy}`;
        await expect(globalStatusAlterationRow.locator(`div:has-text("Horário: ${currentDate}")`)).toBeVisible();
        await expect(globalStatusAlterationRow.locator('div:has-text("Carteira: 0xbcd4...4096")')).toBeVisible();

    });

})