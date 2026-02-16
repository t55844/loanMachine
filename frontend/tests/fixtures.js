import { test as base, expect } from '@playwright/test';

// ==================== FUNÇÕES AUXILIARES ====================

async function connectDemoWalletFunction(page, privateKey = null) {
  const pk = privateKey || '0xf214f2b2cd398c806f84e317254e0f0b801d0643303237d97a22a48e01628897';

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
    page.getByText('Saldo USD:').locator('..')
  ).toContainText(/[1-9]/, { timeout: 30000 });

  await page.getByRole('button', { name: 'Continuar para DApp' }).click({ force: true });

  await expect(page.getByRole('heading', { name: /Loan Machine DApp/i })).toBeVisible();
}

// ==================== FIXTURES ====================

export const test = base.extend({
  // Fixture que permite sobrescrever a chave privada por teste
  privateKey: [
    '0xf214f2b2cd398c806f84e317254e0f0b801d0643303237d97a22a48e01628897', // valor padrão
    { option: true }
  ],

  // Fixture principal de login agora recebe a chave privada
  setupLoggedInUser: async ({ page, privateKey }, use) => {
    await page.goto('/');
    await page.evaluate(() => localStorage.clear());
    await expect(page.getByText('Erro na Conexão')).not.toBeVisible();

    await connectDemoWalletFunction(page, privateKey);
    await getUSDTFromFaucetFunction(page);

    await use();
  },

  // Fixtures auxiliares mantidas
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