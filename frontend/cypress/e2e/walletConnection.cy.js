describe('Wallet Connection Flow', () => {
  beforeEach(() => {
    cy.visit('/');
    cy.clearLocalStorage();  // Clean state

    // Assert no connection/config error – fail early if proxy fetch issue
    cy.contains('Erro na Conexão').should('not.exist');
  });

  it('connects with demo private key, gets USDT from faucet and enters the site', () => {
    // Click demo toggle
    cy.get('button.faucet-button.demo').should('be.visible');
    cy.get('button.faucet-button.demo').click({ force: true });

    // Input private key
    cy.get('input[placeholder="Cole sua chave privada demo (0x...)"]')
      .should('be.visible')
      .type('0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d');

    // Click connect
    cy.get('button.faucet-button.primary:contains("Conectar Demo")')
      .should('be.visible')
      .click({ force: true });

    // Wait for connected state
    cy.contains('Carteira Conectada', { timeout: 20000 }).should('be.visible');
    cy.contains('Endereço:', { timeout: 15000 }).should('be.visible');
    cy.contains('Saldo USDT:', { timeout: 15000 }).should('be.visible');

    cy.get('button.faucet-button')
      .contains('Obter')
      .should('be.visible')
      .click({ force: true });

    cy.get('button.faucet-button')
      .contains('Mintando USDT...', { timeout: 30000 })
      .should('be.visible');

    cy.get('button.faucet-button')
      .contains('Obter', { timeout: 30000 })
      .should('be.visible');

    cy.contains('Saldo USDT:')
      .parent()
      .within(() => {
        cy.contains(/[1-9]\d*(\.\d+)?/).should('be.visible');
      });

    cy.contains('button.faucet-button.primary', 'Continuar para DApp')
      .should('be.visible')
      .click();

    // Main app loaded
    cy.contains('h1', 'Loan Machine DApp').should('be.visible');
    cy.contains('Distribuição de Carteiras').should('be.visible');
  });
});