import { test, expect } from './fixtures.js';

test.describe('Loan system', () => {
    test.beforeEach(async ({ page, setupLoggedInUser }) => {
    //console.log('Running setup before test...');
  });

  test('create an loan requisition', async ({ page, waitForTransaction }) => {
    // Wait for main app to load
    await expect(page.getByRole('heading', { name: /Sistema de Requisição de Empréstimos/i })).toBeVisible({ timeout: 30000 });
    await page.waitForTimeout(3000); // Buffer for rendering


    await page.locator('[pw-test-id="create-requisition-button-tab"]').click({ force: true });

    await expect(page.getByRole('heading', { name: /Criar Requisição de Empréstimo/i })).toBeVisible({ timeout: 10000 });
    await expect(page.locator('label:has-text("Valor do Empréstimo (USD)")')).toBeVisible({ timeout: 10000 });
    await expect(page.locator('label:has-text("Cobertura Mínima (%)")')).toBeVisible({ timeout: 10000 });
    await expect(page.locator('label:has-text("Quantidade de Parcelas")')).toBeVisible({ timeout: 10000 });
    await expect(page.locator('label:has-text("Intervalo de Pagamento (Dias)")')).toBeVisible({ timeout: 10000 });

    await page.locator('select#minimumCoverage').selectOption('71', { force: true , timeout: 5000 });
    await page.locator('select#minimumCoverage').selectOption('75', { force: true , timeout: 5000 });
    await page.locator('select#minimumCoverage').selectOption('80', { force: true , timeout: 5000 });
    await page.locator('select#minimumCoverage').selectOption('85', { force: true , timeout: 5000 });
    await page.locator('select#minimumCoverage').selectOption('90', { force: true , timeout: 5000 });
    await page.locator('select#minimumCoverage').selectOption('95', { force: true , timeout: 5000 });

    await page.locator('select#parcelsQuantity').selectOption('1', { force: true , timeout: 5000 });
    await page.locator('select#parcelsQuantity').selectOption('2', { force: true , timeout: 5000 });
    await page.locator('select#parcelsQuantity').selectOption('3', { force: true , timeout: 5000 });
    await page.locator('select#parcelsQuantity').selectOption('4', { force: true , timeout: 5000 });
    await page.locator('select#parcelsQuantity').selectOption('5', { force: true , timeout: 5000 });
    await page.locator('select#parcelsQuantity').selectOption('6', { force: true , timeout: 5000 });
    await page.locator('select#parcelsQuantity').selectOption('7', { force: true , timeout: 5000 });
    await page.locator('select#parcelsQuantity').selectOption('8', { force: true , timeout: 5000 });
    await page.locator('select#parcelsQuantity').selectOption('9', { force: true , timeout: 5000 });
    await page.locator('select#parcelsQuantity').selectOption('10', { force: true , timeout: 5000 });
    await page.locator('select#parcelsQuantity').selectOption('11', { force: true , timeout: 5000 });

    await page.locator('select#daysIntervalOfPayment').selectOption('1', { force: true , timeout: 5000 });
    await page.locator('select#daysIntervalOfPayment').selectOption('5', { force: true , timeout: 5000 });
    await page.locator('select#daysIntervalOfPayment').selectOption('10', { force: true , timeout: 5000 });
    await page.locator('select#daysIntervalOfPayment').selectOption('15', { force: true , timeout: 5000 });
    await page.locator('select#daysIntervalOfPayment').selectOption('20', { force: true , timeout: 5000 });
    


    await page.locator('input[placeholder="Insira o valor"]').fill('50');
    await page.locator('select#minimumCoverage').selectOption('100', { force: true , timeout: 5000 });
    await page.locator('select#parcelsQuantity').selectOption('12', { force: true , timeout: 5000 });
    await page.locator('select#daysIntervalOfPayment').selectOption('30', { force: true , timeout: 5000 });
    await page.locator('[pw-test-id="submit-create-requisition-button"]').click({ force: true });

    await expect(page.locator('[pw-test-id="toast-notification"]:has-text("Requisição de empréstimo criada com sucesso!")')).toBeVisible({ timeout: 30000 });
  });


 test('check loan requisition previusly created', async ({ page }) => {
    // Wait for main app to load
    await expect(page.getByRole('heading', { name: /Sistema de Requisição de Empréstimos/i })).toBeVisible({ timeout: 30000 });
    await page.waitForTimeout(3000); // Buffer for rendering


    await page.locator('[pw-test-id="monitor-requisitions-button-tab"]').click({ force: true });

    await expect(page.getByRole('heading', { name: /Minhas Requisições de Empréstimo/i })).toBeVisible({ timeout: 10000 });
    await expect(page.getByRole('heading', { name: /Requisição #1/i })).toBeVisible({ timeout: 10000 });

    await expect(page.locator('.requsitionBlock div div span:has-text("Pendente")').first()).toBeVisible({ timeout: 10000 });

    const valorLocator = page.locator('.requsitionBlock div div strong:has-text("Valor:")').first();
    await expect(valorLocator).toBeVisible({ timeout: 10000 });
    await expect(valorLocator.locator('..')).toHaveText('Valor: 50.00 USD', { timeout: 10000 });

    const coverageLocator = page.locator('.requsitionBlock div div strong:has-text("Cobertura:")').first();
    await expect(coverageLocator).toBeVisible({ timeout: 10000 });
    await expect(coverageLocator.locator('..')).toHaveText('Cobertura: 0% / 0%', { timeout: 10000 });

    const creditorsLocator = page.locator('.requsitionBlock div div strong:has-text("Credores:")').first();
    await expect(creditorsLocator).toBeVisible({ timeout: 10000 });
    await expect(creditorsLocator.locator('..')).toHaveText('Credores: 0', { timeout: 10000 });

    const currentDate = new Date().toLocaleDateString('pt-BR');
    const creationTimeLocator = page.locator('.requsitionBlock div div strong:has-text("Criada em:")').first();
    await expect(creationTimeLocator).toBeVisible({ timeout: 10000 });
    await expect(creationTimeLocator.locator('..')).toHaveText(`Criada em: ${currentDate}`, { timeout: 10000 });

    await expect(page.locator('.requsitionBlock div div button:has-text("Cancelar Requisição")').first()).toBeVisible({ timeout: 10000 });
    await page.locator('[pw-test-id="update-requisitions-button"]').click({ force: true });

  });

});