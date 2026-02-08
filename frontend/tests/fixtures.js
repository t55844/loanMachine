import { test as base, expect } from '@playwright/test';

// Create regular helper functions (not fixtures)
async function connectDemoWalletFunction(page, privateKey = null) {
  const pk = privateKey || '0x2a871d0798f97d79848a013d4936a73bf4cc922c825d33c1cf7073dff6d409c6';

  await page.getByRole('button', { name: 'demo' }).click({ force: true });

  await page
    .locator('input[placeholder*="chave privada demo"]')
    .fill(pk);

  await page.getByRole('button', { name: 'Conectar Demo' }).click({ force: true });

  await expect(page.getByText('Carteira Conectada')).toBeVisible({ timeout: 20000 });
}

async function getUSDTFromFaucetFunction(page) {
  const obterButton = page.getByRole('button', { name: 'Obter' });

  for (let i = 0; i < 12; i++) {
    await obterButton.click({ force: true }).catch(() => {});
  }

  await expect(obterButton).toBeVisible({ timeout: 30000 });

  await expect(
    page.getByText('Saldo USDT:').locator('..')
  ).toContainText(/[1-9]/, { timeout: 30000 });

  await page.getByRole('button', { name: 'Continuar para DApp' }).click({ force: true });

  await expect(page.getByRole('heading', { name: /Loan Machine DApp/i })).toBeVisible();
}

export const test = base.extend({
  setupLoggedInUser: async ({ page }, use) => {
    await page.goto('/');
    await page.evaluate(() => localStorage.clear());
    await expect(page.getByText('Erro na Conexão')).not.toBeVisible();

    // ✅ Now call the regular functions
    await connectDemoWalletFunction(page);
    await getUSDTFromFaucetFunction(page);

    await use();
  },

  // Keep these as fixtures for tests that need them individually
  connectDemoWallet: async ({ page }, use) => {
    await use(async (privateKey = null) => {
      await connectDemoWalletFunction(page, privateKey);
    });
  },

  getUSDTFromFaucet: async ({ page }, use) => {
    await use(async () => {
      await getUSDTFromFaucetFunction(page);
    });
  },

  waitForTransaction: async ({ page }, use) => {
    await use(async (timeout = 60000) => {
      await page.waitForSelector(':text("Processando")', { state: 'hidden', timeout }).catch(() => {});
      await page.waitForTimeout(3000);
    });
  },

  extractCurrency: async ({}, use) => {
    await use((text) => parseCurrency(text));
  },
});

function parseCurrency(text) {
  if (!text) return 0;
  const match = text.match(/[\d.,]+/);
  if (!match) return 0;

  let numStr = match[0];
  const dotIndex = numStr.lastIndexOf('.');
  const commaIndex = numStr.lastIndexOf(',');

  let decimalSep = '.';
  let thousandSep = ',';

  if (commaIndex > dotIndex) {
    decimalSep = ',';
    thousandSep = '.';
  }

  if (thousandSep) numStr = numStr.replace(new RegExp('\\' + thousandSep, 'g'), '');
  if (decimalSep === ',') numStr = numStr.replace(',', '.');

  return parseFloat(numStr) || 0;
}

export { parseCurrency };
export { expect };