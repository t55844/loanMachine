// public/privy-bridge.js
import Privy, {
  getUserEmbeddedEthereumWallet,
  getEntropyDetailsFromUser,
} from 'https://esm.sh/@privy-io/js-sdk-core@latest';

// ── Config ─────────────────────────────────────────────────────
const SDK_VERSION  = 'js-sdk-core:0.61.1';
const SESSIONS_URL = 'https://auth.privy.io/api/v1/sessions';

const HARDHAT_LOCAL = {
  id: 31337,
  name: 'Hardhat Local',
  network: 'hardhat',
  nativeCurrency: { name: 'Ether', symbol: 'ETH', decimals: 18 },
  rpcUrls: {
    default: { http: ['http://localhost:8545'] },
    public:  { http: ['http://localhost:8545'] },
  },
};

const getAppId   = () => window.APP_CONFIG?.privyAppId;
const getChainId = () => window.APP_CONFIG?.chainId;
const fire       = (name, detail) =>
  window.dispatchEvent(new CustomEvent(name, { detail }));

// ── Module state ───────────────────────────────────────────────
let privyClient    = null;
let embeddedWallet = null;
let iframeReady    = null;

// ── Storage adapter ────────────────────────────────────────────
const localStorageAdapter = {
  get(key)        { try { return localStorage.getItem(key); }   catch { return null; } },
  put(key, value) { try { localStorage.setItem(key, value); }   catch {} },
  del(key)        { try { localStorage.removeItem(key); }       catch {} },
  getAll() {
    try {
      const out = {};
      for (let i = 0; i < localStorage.length; i++) {
        const k = localStorage.key(i);
        if (k) out[k] = localStorage.getItem(k);
      }
      return out;
    } catch { return {}; }
  },
};

// ── Client + iframe setup (lazy, called once) ──────────────────
function getClient() {
  if (privyClient) return privyClient;

  const appId = getAppId();
  if (!appId) {
    console.error('[privy-bridge] APP_CONFIG.privyAppId missing');
    return null;
  }

  privyClient = new Privy({
    appId,
    storage:         localStorageAdapter,
    supportedChains: [HARDHAT_LOCAL],
    defaultChain:    HARDHAT_LOCAL,
  });

  // Mount the hidden iframe that hosts Privy's wallet enclave.
  const iframe = document.createElement('iframe');
  iframe.src = privyClient.embeddedWallet.getURL();
  iframe.style.cssText = 'display:none;position:absolute;width:0;height:0;border:0';
  document.body.appendChild(iframe);

  iframeReady = new Promise(resolve => {
    iframe.addEventListener('load', () => {
      privyClient.setMessagePoster(iframe.contentWindow);
      resolve();
    });
  });

  // Forward iframe messages to Privy. Filter out browser-extension noise.
  window.addEventListener('message', (e) => {
    if (e.source !== iframe.contentWindow) return;
    if (!e.data || typeof e.data !== 'object') return;
    privyClient.embeddedWallet.onMessage(e.data);
  });

  return privyClient;
}

// ── Build the EIP-1193 provider from a user object ─────────────
async function buildWalletForUser(p, user) {
  await iframeReady;

  let walletMeta = getUserEmbeddedEthereumWallet(user);
  if (!walletMeta) {
    const created = await p.embeddedWallet.create({ recoveryMethod: 'privy' });
    walletMeta = getUserEmbeddedEthereumWallet(created.user);
  }
  if (!walletMeta) throw new Error('failed to get wallet metadata');

  const { entropyId, entropyIdVerifier } = getEntropyDetailsFromUser(user);
  embeddedWallet = await p.embeddedWallet.getEthereumProvider({
    wallet: walletMeta, entropyId, entropyIdVerifier,
  });

  const accounts = await embeddedWallet.request({ method: 'eth_requestAccounts' });
  return accounts[0];
}

// ── Restore session via Privy's REST API ───────────────────────
// The js-sdk-core has no documented session-restore method, so we hit
// /api/v1/sessions directly. Same call the SDK makes internally.
async function tryRestoreSession() {
  const p = getClient();
  if (!p) return null;

  try {
    const token   = await p.getAccessToken();
    const refresh = localStorage.getItem('privy:refresh_token');
    if (!token || !refresh) return null;

    const resp = await fetch(SESSIONS_URL, {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type':  'application/json',
        'privy-app-id':  getAppId(),
        'privy-ca-id':   localStorage.getItem('privy:caid') ?? '',
        'privy-client':  SDK_VERSION,
      },
      body: JSON.stringify({ refresh_token: refresh }),
    });

    if (!resp.ok) return null;

    const { user } = await resp.json();
    if (!user || !Array.isArray(user.linked_accounts)) return null;

    return await buildWalletForUser(p, user);
  } catch {
    return null;
  }
}

// ── Email + OTP overlays ──────────────────────────────────────
const OVERLAY_BASE_CSS =
  'position:fixed;inset:0;background:rgba(0,0,0,0.85);display:flex;' +
  'align-items:center;justify-content:center;z-index:9999;' +
  "font-family:'IBM Plex Mono',monospace";

const CARD_BASE_CSS =
  'background:#1a1a1a;border:1px solid rgba(240,204,0,0.3);' +
  'border-radius:12px;padding:32px;width:380px;max-width:90vw';

function showEmailOverlay() {
  return new Promise((resolve, reject) => {
    const overlay = document.createElement('div');
    overlay.style.cssText = OVERLAY_BASE_CSS;
    overlay.innerHTML = `
      <div style="${CARD_BASE_CSS}">
        <h2 style="color:#f0cc00;font-family:'Bebas Neue',sans-serif;font-size:1.8rem;margin-bottom:8px;letter-spacing:0.05em">CONECTAR</h2>
        <p style="color:#888;font-size:0.78rem;margin-bottom:20px">Privy cria sua carteira via e-mail. Sem seed phrase.</p>
        <input id="pb-email" type="email" placeholder="seu@email.com"
          style="width:100%;background:#2a2a2a;color:#f0ebe0;border:1px solid #444;border-radius:6px;padding:10px 14px;font-family:inherit;font-size:0.95rem;outline:none;box-sizing:border-box"/>
        <div style="display:flex;gap:10px;margin-top:14px">
          <button id="pb-cancel" style="flex:1;background:transparent;color:#888;border:1px solid #444;border-radius:6px;padding:10px;cursor:pointer;font-family:inherit">Cancelar</button>
          <button id="pb-submit" style="flex:2;background:#f0cc00;color:#111;border:none;border-radius:6px;padding:10px;cursor:pointer;font-family:'Bebas Neue',sans-serif;font-size:1rem;letter-spacing:0.05em">CONTINUAR</button>
        </div>
        <p id="pb-err" style="color:#e74c3c;font-size:0.75rem;margin-top:10px;display:none"></p>
      </div>`;
    document.body.appendChild(overlay);

    const $ = (sel) => overlay.querySelector(sel);
    const close = () => document.body.removeChild(overlay);

    $('#pb-cancel').onclick = () => { close(); reject(new Error('cancelled')); };
    $('#pb-submit').onclick = () => {
      const email = $('#pb-email').value.trim();
      if (!email.includes('@')) {
        $('#pb-err').textContent = 'E-mail inválido.';
        $('#pb-err').style.display = 'block';
        return;
      }
      close(); resolve(email);
    };
    $('#pb-email').onkeydown = e => { if (e.key === 'Enter') $('#pb-submit').click(); };
    $('#pb-email').focus();
  });
}

function showOtpOverlay(email) {
  return new Promise((resolve, reject) => {
    const overlay = document.createElement('div');
    overlay.style.cssText = OVERLAY_BASE_CSS;
    overlay.innerHTML = `
      <div style="${CARD_BASE_CSS}">
        <h2 style="color:#f0cc00;font-family:'Bebas Neue',sans-serif;font-size:1.8rem;margin-bottom:8px">CÓDIGO</h2>
        <p style="color:#888;font-size:0.78rem;margin-bottom:20px">Enviamos um código para <strong style="color:#f0ebe0">${email.slice(0,3)}***@${email.split('@')[1] ?? ''}</strong></p>
        <input id="pb-otp" type="text" placeholder="123456" maxlength="6"
          style="width:100%;background:#2a2a2a;color:#f0cc00;border:1px solid #444;border-radius:6px;padding:10px 14px;font-family:inherit;font-size:1.4rem;letter-spacing:0.3em;text-align:center;outline:none;box-sizing:border-box"/>
        <div style="display:flex;gap:10px;margin-top:14px">
          <button id="pb-back" style="flex:1;background:transparent;color:#888;border:1px solid #444;border-radius:6px;padding:10px;cursor:pointer;font-family:inherit">Voltar</button>
          <button id="pb-verify" style="flex:2;background:#f0cc00;color:#111;border:none;border-radius:6px;padding:10px;cursor:pointer;font-family:'Bebas Neue',sans-serif;font-size:1rem;letter-spacing:0.05em">VERIFICAR</button>
        </div>
        <p id="pb-err" style="color:#e74c3c;font-size:0.75rem;margin-top:10px;display:none"></p>
      </div>`;
    document.body.appendChild(overlay);

    const $ = (sel) => overlay.querySelector(sel);
    const close = () => document.body.removeChild(overlay);

    $('#pb-back').onclick = () => { close(); reject(new Error('back')); };
    $('#pb-verify').onclick = () => {
      const code = $('#pb-otp').value.trim();
      if (code.length < 6) {
        $('#pb-err').textContent = 'Código deve ter 6 dígitos.';
        $('#pb-err').style.display = 'block';
        return;
      }
      close(); resolve({ code });
    };
    $('#pb-otp').onkeydown = e => { if (e.key === 'Enter') $('#pb-verify').click(); };
    $('#pb-otp').focus();
  });
}

// ── Public window API ──────────────────────────────────────────

window.loan_machine_try_restore = async function () {
  const address = await tryRestoreSession();
  if (address) fire('privy_wallet_ready', { address });
};

window.loan_machine_init_privy = async function () {
  try {
    const restored = await tryRestoreSession();
    if (restored) { fire('privy_wallet_ready', { address: restored }); return; }

    const p = getClient();
    if (!p) throw new Error('client creation failed');

    const email = await showEmailOverlay();
    await p.auth.email.sendCode(email);
    const { code } = await showOtpOverlay(email);
    const { user } = await p.auth.email.loginWithCode(email, code);

    const address = await buildWalletForUser(p, user);
    fire('privy_wallet_ready', { address });
  } catch (err) {
    if (err.message === 'cancelled' || err.message === 'back') return;
    console.error('[privy-bridge] auth error:', err);
    fire('privy_auth_error', { error: err.message });
  }
};

window.loan_machine_get_access_token = async function () {
  try { return await getClient()?.getAccessToken() ?? null; }
  catch { return null; }
};

window.loan_machine_send_tx = async function (txJson) {
  try {
    if (!embeddedWallet) throw new Error('wallet not ready');
    const tx = JSON.parse(txJson);
    const txHash = await embeddedWallet.request({
      method: 'eth_sendTransaction',
      params: [{
        to:      tx.to,
        data:    tx.data,
        value:   tx.value ?? '0x0',
        gas:     tx.gas,
        chainId: '0x' + getChainId().toString(16),
      }],
    });
    fire('privy_tx_complete', { tx_hash: txHash });
  } catch (err) {
    console.error('[privy-bridge] tx error:', err);
    fire('privy_tx_error', { error: err.message });
  }
};

window.loan_machine_logout = async function () {
  try { await getClient()?.auth.logout(); } catch {}
  embeddedWallet = null;
  fire('privy_logged_out', {});
};

window.__privy_bridge_ready = true;