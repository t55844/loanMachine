import { test, expect } from './fixtures.js';


test.describe('Eelection creation and voting', () => {
    test.beforeEach(async ({ page, setupLoggedInUser }) => {
    //console.log('Running setup before test...');
  });

    test('should create an election with valid member IDs', async ({ page, waitForTransaction }) => {
    // Wait for main app to load
    await expect(page.getByRole('heading', { name: /Sistema de Requisição de Empréstimos/i })).toBeVisible({ timeout: 30000 });
    await page.waitForTimeout(3000); // Buffer for rendering

    // Navigate to "Criar Eleição" tab
    // The tab is inside ElectionManagement; we click the button with that name
    await page.getByRole('button', { name: 'Criar Eleição' }).click({ force: true });

    // Verify all text and titles in the CreateElection component
    await expect(page.getByRole('heading', { name: /Criar Nova Eleição/i })).toBeVisible({ timeout: 10000 });

    const createElectionSction = page.locator('[pw-test-id="create-election-component"]');

    await expect(createElectionSction.locator('p:has-text("Crie uma nova eleição fornecendo os IDs de membro do candidato e do oponente.")')).toBeVisible({ timeout: 10000 });

    // Check that the member vinculated status is shown (assumes test account is vinculated)
    // This verifies that the component loads with proper member data
    await expect(createElectionSction.locator('div:has-text("Status do Membro: Vinculado como Membro #")')).toBeVisible();

    // Verify the descriptive text
    await expect(createElectionSction.locator('p:has-text("Crie uma nova eleição fornecendo os IDs de membro do candidato e do oponente.")')).toBeVisible();

    // Check input placeholders
    const candidateInput = page.locator('input[placeholder="ID do Membro Candidato"]');
    const opponentInput = page.locator('input[placeholder="ID do Membro Oponente"]');
    await expect(candidateInput).toBeVisible();
    await expect(opponentInput).toBeVisible();

    // The "Criar Eleição" button should be disabled initially because inputs are empty
    const createButton = createElectionSction.locator('button:has-text("Criar Eleição")');
    await expect(createButton).toBeDisabled();

    // Fill the candidate and opponent IDs
    await candidateInput.fill('321654987');
    await opponentInput.fill('987654321');

    // Now the button should become enabled
    await expect(createButton).toBeEnabled();

    // Click the button to create the election
    await createButton.click();

    // Wait for the success toast notification
    //await expect(createElectionSction.locator('[pw-test-id="toast-notification"]:has-text("Eleição criada com sucesso!")')).toBeVisible({ timeout: 30000 });

    });


  test('should display correct election information before voting', async ({ page }) => {
    // Wait for main app to load
    await expect(page.getByRole('heading', { name: /Sistema de Requisição de Empréstimos/i })).toBeVisible({ timeout: 30000 });
    await page.waitForTimeout(3000); // Buffer for rendering

    // Navigate to "Votar & Candidatos" tab (election already created by previous test)
    await page.getByRole('button', { name: 'Votar & Candidatos' }).click({ force: true });

    // Wait for heading
    await expect(page.getByRole('heading', { name: /Gerenciamento de Eleição/i })).toBeVisible({ timeout: 10000 });

    const electionVoteSection = page.locator('[pw-test-id="election-section"]');

    // Wait for active election heading (since we just created one)
    const activeElectionHeading = electionVoteSection.getByRole('heading', { name: /Eleição Ativa #/i });
    await expect(activeElectionHeading).toBeVisible({ timeout: 15000 });

    // Verify election ID is a positive number (e.g., #1, #2, ...)
    const electionIdText = await activeElectionHeading.textContent();
    const electionIdMatch = electionIdText.match(/#(\d+)/);
    expect(electionIdMatch).toBeTruthy();


    // Check candidate list contains both IDs used in creation (321654987 and 987654321)
    const candidate321 = electionVoteSection.locator('label:has-text("Membro #321654987")');
    const candidate987 = electionVoteSection.locator('label:has-text("Membro #987654321")');
    await expect(candidate321).toBeVisible({ timeout: 5000 });
    await expect(candidate987).toBeVisible({ timeout: 5000 });

    // Verify total votes is 0 initially
    const totalVotesLocator = electionVoteSection.locator('.stats-grid div:has-text("Total de Votos:")');
    await expect(totalVotesLocator).toContainText('0', { timeout: 5000 });

    // Check that start and end times are displayed
    await expect(electionVoteSection.locator('.stats-grid div:has-text("Início:")')).toBeVisible();
    await expect(electionVoteSection.locator('.stats-grid div:has-text("Término:")')).toBeVisible();

    // Verify voting section exists
    const voteSection = electionVoteSection.locator('.vinculate-section:has-text("Emitir Seu Voto")');
    await expect(voteSection).toBeVisible();

    // Verify add candidate section exists
    const addCandidateSection = electionVoteSection.locator('.vinculate-section:has-text("Adicionar Candidato")');
    await expect(addCandidateSection).toBeVisible();

    // Ensure no error toasts are present
    const errorToast = page.locator('[pw-test-id="toast-notification"][data-type="error"]');
    await expect(errorToast).not.toBeVisible({ timeout: 2000 });
  });

  test('should vote in the election and update UI', async ({ page, waitForTransaction }) => {
    // Navigate to voting tab if not already there (page is shared)
    await page.getByRole('button', { name: 'Votar & Candidatos' }).click({ force: true });

    // Wait for active election to be visible
    const activeElectionHeading = page.getByRole('heading', { name: /Eleição Ativa #/i });
    await expect(activeElectionHeading).toBeVisible({ timeout: 15000 });

    // Select the first candidate (321654987)
    const candidateRadio = page.locator('input[type="radio"][value="321654987"]');
    await expect(candidateRadio).toBeVisible();
    await candidateRadio.check();

    // Click the "Emitir Voto" button
    const voteButton = page.locator('button.confirm-button:has-text("Emitir Voto")');
    await expect(voteButton).toBeEnabled();
    await voteButton.click();

    // Wait for success toast
    await expect(page.locator('[pw-test-id="toast-notification"]:has-text("Voto emitido com sucesso!")')).toBeVisible({ timeout: 30000 });

    // Wait for blockchain confirmation
    await waitForTransaction(45000);

    // After voting, the election info should refresh. We'll check that total votes increased to 1.
    //const totalVotesLocator = page.locator('.stats-grid div:has-text("Total de Votos:")');
    //await expect(totalVotesLocator).toContainText('1', { timeout: 10000 });

    // Optionally, check that the radio button is cleared or still selected (implementation dependent)
    // We can verify that the selectedCandidate state might be cleared, so radio is unchecked
    //await expect(candidateRadio).not.toBeChecked();
  });

    test('should display updated election results after voting', async ({ page }) => {
        // Wait for main app to load
        await expect(page.getByRole('heading', { name: /Sistema de Requisição de Empréstimos/i })).toBeVisible({ timeout: 30000 });
        await page.waitForTimeout(3000); // Buffer for rendering

        // Navigate to "Votar & Candidatos" tab
        await page.getByRole('button', { name: 'Votar & Candidatos' }).click({ force: true });

        const electionSection = page.locator('[pw-test-id="election-section"]');

        // Wait for the active election to end and the results to appear.
        // This uses a retry loop: as long as the active election heading is visible,
        // we wait and reload (or just wait). Once it disappears, the winner banner should be visible.
        await expect(async () => {
            // Check if there's an active election heading
            const activeHeading = electionSection.getByRole('heading', { name: /Eleição Ativa #/i });
            const isActiveVisible = await activeHeading.isVisible().catch(() => false);
            if (isActiveVisible) {
            // Still active – force a refresh of the component data (or just wait)
            // In a real scenario you might need to wait for the contract's end time.
            // Here we simply reload the page to fetch updated blockchain state.
            await page.reload();
            await page.waitForTimeout(3000);
            // Re-locate the election section after reload
            await page.getByRole('button', { name: 'Votar & Candidatos' }).click({ force: true });
            throw new Error('Election still active, retrying...');
            }

            // Now verify the winner banner is present
            const winnerBanner = electionSection.locator('h3:has-text("🏆 VENCEDOR DA ÚLTIMA ELEIÇÃO")');
            await expect(winnerBanner).toBeVisible();
        }).toPass({ timeout: 60000, intervals: [5000] }); // Wait up to 60 seconds, polling every 5s

        // ---- Assertions on the result details ----
        // Winner member ID (should match the candidate we voted for in the previous test)
        await expect(electionSection.locator('h4:has-text("Membro #321654987")')).toBeVisible();

        // Winning vote count – at least 1, but we can check the exact number if known
        const winningVotesText = await electionSection.locator('h4:has-text("com")').textContent();
        expect(winningVotesText).toMatch(/com \d+ votos/);

        // Verify the stats grid
        const statsGrid = electionSection.locator('.stats-grid');
        await expect(statsGrid.locator('div:has-text("Vencedor:")')).toContainText('Membro #321654987');
        await expect(statsGrid.locator('div:has-text("Votos Vencedores:")')).toContainText(/\d+/);
        await expect(statsGrid.locator('div:has-text("ID da Eleição:")')).toBeVisible();
        await expect(statsGrid.locator('div:has-text("Encerrada Em:")')).toBeVisible();

        // Ensure no active election is shown
        await expect(electionSection.getByRole('heading', { name: /Eleição Ativa #/i })).not.toBeVisible();
    });
  
});
