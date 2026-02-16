import { test, expect } from './fixtures.js';


const testCases = [
  {
    name: 'Default Key',
    privateKey: '0xf214f2b2cd398c806f84e317254e0f0b801d0643303237d97a22a48e01628897',
    memberId: '321654987'
  },
  {
    name: 'Alternative Key',
    privateKey: '0x2a871d0798f97d79848a013d4936a73bf4cc922c825d33c1cf7073dff6d409c6', 
    memberId: '987654321'                    
  }
];

for (const { name, privateKey, memberId } of testCases) {
  test.describe(`Wallet Vinculation - ${name}`, () => {
    // Aplica a chave privada específica para este grupo de testes
    test.use({ privateKey });

    test.beforeEach(async ({ page, setupLoggedInUser }) => {
      // setupLoggedInUser já usa a chave definida acima
    });

    test(`should link member ID ${memberId} successfully`, async ({ page, waitForTransaction }) => {
      await expect(page.getByRole('heading', { name: 'Loan Machine DApp' })).toBeVisible({ timeout: 15000 });

      await page.waitForTimeout(2000);

      await page.locator('button.menu-toggle.left').click({ timeout: 10000 });
      await expect(page.locator('.side-menu.left.visible')).toBeVisible({ timeout: 10000 });

      // Preenche o Member ID (diferente em cada execução)
      await page
        .locator('.side-menu.left.visible [data-cy="member-id-input"]')
        .fill(memberId);

      await page
        .locator('button.vinculate-button:has-text("Vincular Membro")')
        .scrollIntoViewIfNeeded();

      await page
        .locator('button.vinculate-button:has-text("Vincular Membro")')
        .click({ force: true, timeout: 10000 });

      // Confirma transação
      await page
        .locator('.confirmation-modal-overlay')
        .getByRole('button', { name: 'Confirmar Transação' })
        .click();

      await expect(page.locator('.confirmation-modal-overlay')).not.toBeVisible({ timeout: 20000 });

      await waitForTransaction(45000);

      // Tenta novamente (banner)
      await page
        .locator('.wallet-verification-banner .retry-button-banner')
        .click({ force: true });

      await expect(page.locator('.wallet-verification-banner')).not.toBeVisible({ timeout: 25000 });

      await page.waitForTimeout(5000);

      // Abre sidebar novamente e verifica o Member ID
      const menuButton = page.locator('button.menu-toggle.left');
      const sidebar = page.locator('.side-menu.left.visible');

      await menuButton.click({ force: true });
      await page.waitForTimeout(1000);

      await expect(sidebar).toBeVisible({ timeout: 15000 });
      await page.waitForTimeout(2000);

      const hasMemberId = await sidebar.locator(`:has-text("${memberId}")`).count() > 0;

      if (!hasMemberId) {
        await page.screenshot({ path: `test-results/error-no-member-id-${memberId}.png` });
        throw new Error(`Member ID ${memberId} não foi exibido após vinculação`);
      }
    });
  });
}