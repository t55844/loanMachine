import { test, expect } from '@playwright/test';

test.describe('Wallet Connection Flow', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.evaluate(() => localStorage.clear());  // Clean state

    // Assert no connection/config error – fail early if proxy fetch issue
    await expect(page.getByText('Erro na Conexão')).not.toBeVisible();
  });

  test('connects with demo private key, gets USDT from faucet and enters the site', async ({ page }) => {
    // Click demo toggle
    const demoButton = page.getByRole('button', { name: 'demo' }); // Ajuste se o texto exato for diferente, mas classe sugere isso
    await expect(demoButton).toBeVisible();
    await demoButton.click({ force: true });

    // Input private key
    const privateKeyInput = page.locator('input[placeholder="Cole sua chave privada demo (0x...)"]');
    await expect(privateKeyInput).toBeVisible();
    await privateKeyInput.fill('0xdbda1821b80551c9d65939329250298aa3472ba22feea921c0cf5d620ea67b97');

    // Click connect
    const connectButton = page.getByRole('button', { name: 'Conectar Demo' });
    await expect(connectButton).toBeVisible();
    await connectButton.click({ force: true });

    // Wait for connected state
    await expect(page.getByText('Carteira Conectada')).toBeVisible({ timeout: 20000 });
    await expect(page.getByText('Endereço:')).toBeVisible({ timeout: 15000 });
    await expect(page.getByText('Saldo USDT:')).toBeVisible({ timeout: 15000 });

    // Faucet button "Obter"
    const faucetButton = page.getByRole('button', { name: 'Obter' });
    await expect(faucetButton).toBeVisible();
    await faucetButton.click({ force: true });

    await expect(page.getByRole('button', { name: 'Mintando USDT...' })).toBeVisible({ timeout: 30000 });

    await expect(page.getByRole('button', { name: 'Obter' })).toBeVisible({ timeout: 30000 });

    await expect(page.locator('text=Saldo USDT:').locator('..').getByText(/[1-9]\d*(\.\d+)?/)).toBeVisible();

    // Continue button
    const continueButton = page.getByRole('button', { name: 'Continuar para DApp' });
    await expect(continueButton).toBeVisible();
    await continueButton.click();

    // Main app loaded
    await expect(page.getByRole('heading', { name: 'Loan Machine DApp' })).toBeVisible();
    await expect(page.getByText('Distribuição de Carteiras')).toBeVisible();
  });
});