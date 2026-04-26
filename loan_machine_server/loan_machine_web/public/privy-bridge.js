// public/privy-bridge.js
import Privy, {
  getUserEmbeddedEthereumWallet,
  getEntropyDetailsFromUser,
} from 'https://esm.sh/@privy-io/js-sdk-core@latest';

function getAppId()   { return window.APP_CONFIG?.privyAppId; }
function getChainId() { return window.APP_CONFIG?.chainId; }

const DEBUG = window.APP_CONFIG?.debug === true;
const log   = (...args) => { if (DEBUG) console.log('[privy-bridge]', ...args); };

console.log('[privy-bridge] module parsed OK, Privy:', typeof Privy);

let privyClient    = null;
let embeddedWallet = null;
let iframeReady    = null;

// ── localStorage adapter ───────────────────────────────────────
const localStorageAdapter = {
  get(key) {
    try { return localStorage.getItem(key); }
    catch (e) { console.warn('[storage] get failed:', e); return null; }
  },
  put(key, value) {
    try { localStorage.setItem(key, value); }
    catch (e) { console.warn('[storage] put failed:', e); }
  },
  del(key) {
    try { localStorage.removeItem(key); }
    catch (e) { console.warn('[storage] del failed:', e); }
  },
  getAll() {
    try {
      const result = {};
      for (let i = 0; i < localStorage.length; i++) {
        const k = localStorage.key(i);
        if (k) result[k] = localStorage.getItem(k);
      }
      return result;
    } catch { return {}; }
  },
};

// ── Client + iframe setup ──────────────────────────────────────
function getClient() {
  if (!privyClient) {
    const appId = getAppId();
    if (!appId) { console.error('[privy-bridge] APP_CONFIG missing'); return null; }

    privyClient = new Privy({
      appId,
      storage: localStorageAdapter,
      supportedChains: [{
        id: 31337,
        name: 'Hardhat Local',
        network: 'hardhat',
        nativeCurrency: { name: 'Ether', symbol: 'ETH', decimals: 18 },
        rpcUrls: {
          default: { http: ['http://localhost:8545'] },
          public:  { http: ['http://localhost:8545'] },
        },
      }],
      defaultChain: {
        id: 31337,
        name: 'Hardhat Local',
        network: 'hardhat',
        nativeCurrency: { name: 'Ether', symbol: 'ETH', decimals: 18 },
        rpcUrls: {
          default: { http: ['http://localhost:8545'] },
          public:  { http: ['http://localhost:8545'] },
        },
      },
    });

    // Mount the hidden iframe that hosts Privy's wallet enclave.
    // Without this, embeddedWallet.create() throws "proxy not initialized".
    const iframeUrl = privyClient.embeddedWallet.getURL();
    const iframe    = document.createElement('iframe');
    iframe.src      = iframeUrl;
    iframe.style.cssText = 'display:none; position:absolute; width:0; height:0; border:0;';
    document.body.appendChild(iframe);

    // Resolve when the iframe has loaded and the message channel is open
    iframeReady = new Promise((resolve) => {
      iframe.addEventListener('load', () => {
        privyClient.setMessagePoster(iframe.contentWindow);
        log('iframe mounted + message poster set');
        resolve();
      });
    });

    // Forward messages FROM the iframe to Privy.
    // Filter out MetaMask / extension noise by checking the source.
    window.addEventListener('message', (e) => {
      if (e.source !== iframe.contentWindow) return;
      if (!e.data || typeof e.data !== 'object') return;
      privyClient.embeddedWallet.onMessage(e.data);
    });
  }
  return privyClient;
}

function fire(name, detail) {
  window.dispatchEvent(new CustomEvent(name, { detail }));
}

// ── Email overlay ──────────────────────────────────────────────
function showEmailOverlay() {
  return new Promise((resolve, reject) => {
    const overlay = document.createElement('div');
    overlay.style.cssText = `position:fixed;inset:0;background:rgba(0,0,0,0.85);display:flex;align-items:center;justify-content:center;z-index:9999;font-family:'IBM Plex Mono',monospace;`;
    overlay.innerHTML = `
      <div style="background:#1a1a1a;border:1px solid rgba(240,204,0,0.3);border-radius:12px;padding:32px;width:380px;max-width:90vw;">
        <h2 style="color:#f0cc00;font-family:'Bebas Neue',sans-serif;font-size:1.8rem;margin-bottom:8px;letter-spacing:0.05em;">CONECTAR</h2>
        <p style="color:#888;font-size:0.78rem;margin-bottom:20px;">Privy cria sua carteira via e-mail. Sem seed phrase.</p>
        <input id="pb-email" type="email" placeholder="seu@email.com"
          style="width:100%;background:#2a2a2a;color:#f0ebe0;border:1px solid #444;border-radius:6px;padding:10px 14px;font-family:inherit;font-size:0.95rem;outline:none;box-sizing:border-box;"/>
        <div style="display:flex;gap:10px;margin-top:14px;">
          <button id="pb-cancel" style="flex:1;background:transparent;color:#888;border:1px solid #444;border-radius:6px;padding:10px;cursor:pointer;font-family:inherit;">Cancelar</button>
          <button id="pb-submit" style="flex:2;background:#f0cc00;color:#111;border:none;border-radius:6px;padding:10px;cursor:pointer;font-family:'Bebas Neue',sans-serif;font-size:1rem;letter-spacing:0.05em;">CONTINUAR</button>
        </div>
        <p id="pb-err" style="color:#e74c3c;font-size:0.75rem;margin-top:10px;display:none;"></p>
      </div>`;
    document.body.appendChild(overlay);
    const input  = overlay.querySelector('#pb-email');
    const submit = overlay.querySelector('#pb-submit');
    const cancel = overlay.querySelector('#pb-cancel');
    const err    = overlay.querySelector('#pb-err');
    const close  = () => document.body.removeChild(overlay);
    cancel.onclick = () => { close(); reject(new Error('cancelled')); };
    submit.onclick = () => {
      const email = input.value.trim();
      if (!email.includes('@')) { err.textContent = 'E-mail inválido.'; err.style.display = 'block'; return; }
      close(); resolve(email);
    };
    input.onkeydown = e => { if (e.key === 'Enter') submit.click(); };
    input.focus();
  });
}

// ── OTP overlay ────────────────────────────────────────────────
function showOtpOverlay(email) {
  return new Promise((resolve, reject) => {
    const overlay = document.createElement('div');
    overlay.style.cssText = `position:fixed;inset:0;background:rgba(0,0,0,0.85);display:flex;align-items:center;justify-content:center;z-index:9999;font-family:'IBM Plex Mono',monospace;`;
    overlay.innerHTML = `
      <div style="background:#1a1a1a;border:1px solid rgba(240,204,0,0.3);border-radius:12px;padding:32px;width:380px;max-width:90vw;">
        <h2 style="color:#f0cc00;font-family:'Bebas Neue',sans-serif;font-size:1.8rem;margin-bottom:8px;">CÓDIGO</h2>
        <p style="color:#888;font-size:0.78rem;margin-bottom:20px;">Enviamos um código para <strong style="color:#f0ebe0;">${email.slice(0, 3)}***@${email.split('@')[1] ?? ''}</strong></p>
        <input id="pb-otp" type="text" placeholder="123456" maxlength="6"
          style="width:100%;background:#2a2a2a;color:#f0cc00;border:1px solid #444;border-radius:6px;padding:10px 14px;font-family:inherit;font-size:1.4rem;letter-spacing:0.3em;text-align:center;outline:none;box-sizing:border-box;"/>
        <div style="display:flex;gap:10px;margin-top:14px;">
          <button id="pb-back" style="flex:1;background:transparent;color:#888;border:1px solid #444;border-radius:6px;padding:10px;cursor:pointer;font-family:inherit;">Voltar</button>
          <button id="pb-verify" style="flex:2;background:#f0cc00;color:#111;border:none;border-radius:6px;padding:10px;cursor:pointer;font-family:'Bebas Neue',sans-serif;font-size:1rem;letter-spacing:0.05em;">VERIFICAR</button>
        </div>
        <p id="pb-err" style="color:#e74c3c;font-size:0.75rem;margin-top:10px;display:none;"></p>
      </div>`;
    document.body.appendChild(overlay);
    const input  = overlay.querySelector('#pb-otp');
    const verify = overlay.querySelector('#pb-verify');
    const back   = overlay.querySelector('#pb-back');
    const err    = overlay.querySelector('#pb-err');
    const close  = () => document.body.removeChild(overlay);
    back.onclick   = () => { close(); reject(new Error('back')); };
    verify.onclick = () => {
      const code = input.value.trim();
      if (code.length < 6) { err.textContent = 'Código deve ter 6 dígitos.'; err.style.display = 'block'; return; }
      close(); resolve({ code });
    };
    input.onkeydown = e => { if (e.key === 'Enter') verify.click(); };
    input.focus();
  });
}

// ── Main auth flow ─────────────────────────────────────────────
window.loan_machine_init_privy = async function () {
  log('loan_machine_init_privy called');
  try {
    const p = getClient();
    if (!p) throw new Error('client creation failed');

    const email = await showEmailOverlay();

    log('calling sendCode...');
    await p.auth.email.sendCode(email);
    log('OTP sent');

    const { code } = await showOtpOverlay(email);
    log('calling loginWithCode...');

    const { user } = await p.auth.email.loginWithCode(email, code);
    log('logged in:', user.id.slice(0, 15) + '...');

    await iframeReady;

    // ── Get or create the embedded wallet metadata ─────────────
    let walletMeta = getUserEmbeddedEthereumWallet(user);
    if (!walletMeta) {
      log('creating new wallet...');
      const createResult = await p.embeddedWallet.create({ recoveryMethod: 'privy' });
      walletMeta = getUserEmbeddedEthereumWallet(createResult.user);
    } else {
      log('found existing wallet');
    }

    if (!walletMeta) throw new Error('failed to get wallet metadata');

    // ── Get the actual EIP-1193 provider ───────────────────────
    const { entropyId, entropyIdVerifier } = getEntropyDetailsFromUser(user);
    const provider = await p.embeddedWallet.getEthereumProvider({
      wallet: walletMeta,
      entropyId,
      entropyIdVerifier,
    });
    embeddedWallet = provider;

    const accounts = await provider.request({ method: 'eth_requestAccounts' });
    log('wallet ready:', accounts[0]);
    fire('privy_wallet_ready', { address: accounts[0] });

  } catch (err) {
    if (err.message === 'cancelled' || err.message === 'back') return;
    console.error('[privy-bridge] auth error:', err);
    fire('privy_auth_error', { error: err.message });
  }
};

// ── Send transaction ───────────────────────────────────────────
window.loan_machine_send_tx = async function (txJson) {
  try {
    if (!embeddedWallet) throw new Error('wallet not ready');
    const tx = JSON.parse(txJson);
    const txHash = await embeddedWallet.request({
      method: 'eth_sendTransaction',
      params: [{ to: tx.to, data: tx.data, value: tx.value ?? '0x0', gas: tx.gas,
                 chainId: '0x' + getChainId().toString(16) }],
    });
    fire('privy_tx_complete', { tx_hash: txHash });
  } catch (err) {
    console.error('[privy-bridge] tx error:', err);
    fire('privy_tx_error', { error: err.message });
  }
};

// ── Logout ─────────────────────────────────────────────────────
window.loan_machine_logout = async function () {
  try {
    await getClient()?.auth.logout();
    embeddedWallet = null;
    fire('privy_logged_out', {});
  } catch {
    fire('privy_logged_out', {});
  }
};

window.__privy_bridge_ready = true;