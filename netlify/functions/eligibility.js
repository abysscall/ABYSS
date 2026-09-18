// netlify/functions/eligibility.js
//
// Server-side eligibility decision for the ABYSS Gateway (ADR-0025).
// The browser never decides ALLOWED/DENIED -- this function does, and it is
// the only place the restricted-code list lives. Nothing here is visible
// to client-side JavaScript or page source.
//
// Two actions, one endpoint:
//   { action: "check",   countryCode: "+420" }
//     -> { result: "ELIGIBLE" | "RESTRICTED" }
//     -> on ELIGIBLE, also sets a short-lived signed "pending" cookie
//
//   { action: "declare", declarationVersion: "ETD-001-1.0", accepted: true }
//     -> reads the pending cookie set by "check", verifies it server-side,
//        and only then issues the real access cookie (abyss_gate).
//        A "declare" call without a valid pending cookie is rejected.
//
// Required environment variable (Netlify dashboard -> Site settings ->
// Environment variables): ABYSS_GATE_SECRET -- any long random string.

const crypto = require('crypto');

// Placeholder restricted list -- country-code digits without "+".
// This is NOT a confirmed legal jurisdiction list. See ADR-0025 Open
// Questions. Replace only once counsel confirms the actual restricted
// set, and note it may differ per ABYSS function rather than being a
// single global list.
const RESTRICTED_CODES = ['420', '49', '33'];

const PENDING_MAX_AGE = 60 * 10;        // 10 minutes to complete the declaration
const GATE_MAX_AGE = 60 * 60 * 24 * 30; // 30 days

function base64url(buf) {
  return buf.toString('base64').replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');
}

function sign(payloadObj, secret) {
  const payload = base64url(Buffer.from(JSON.stringify(payloadObj)));
  const sig = base64url(crypto.createHmac('sha256', secret).update(payload).digest());
  return `${payload}.${sig}`;
}

function verify(token, secret) {
  if (!token || typeof token !== 'string' || !token.includes('.')) return null;
  const [payload, sig] = token.split('.');
  const expected = base64url(crypto.createHmac('sha256', secret).update(payload).digest());
  if (expected !== sig) return null;
  try {
    return JSON.parse(Buffer.from(payload.replace(/-/g, '+').replace(/_/g, '/'), 'base64').toString());
  } catch (e) {
    return null;
  }
}

function parseCookies(header) {
  const out = {};
  (header || '').split(';').forEach((pair) => {
    const idx = pair.indexOf('=');
    if (idx === -1) return;
    out[pair.slice(0, idx).trim()] = decodeURIComponent(pair.slice(idx + 1).trim());
  });
  return out;
}

exports.handler = async (event) => {
  const secret = process.env.ABYSS_GATE_SECRET;
  if (!secret) {
    return { statusCode: 500, body: JSON.stringify({ error: 'Server misconfigured: ABYSS_GATE_SECRET not set' }) };
  }
  if (event.httpMethod !== 'POST') {
    return { statusCode: 405, body: JSON.stringify({ error: 'Method not allowed' }) };
  }

  let body;
  try {
    body = JSON.parse(event.body || '{}');
  } catch (e) {
    return { statusCode: 400, body: JSON.stringify({ error: 'Invalid JSON body' }) };
  }

  // ---- action: check ----
  if (body.action === 'check') {
    const raw = String(body.countryCode || '').replace('+', '').trim();
    if (!raw) {
      return { statusCode: 400, body: JSON.stringify({ error: 'countryCode required' }) };
    }
    const restricted = RESTRICTED_CODES.includes(raw);
    const result = restricted ? 'RESTRICTED' : 'ELIGIBLE';

    if (restricted) {
      return { statusCode: 200, headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ result }) };
    }

    const pending = sign({ r: 'ELIGIBLE', t: Date.now() }, secret);
    return {
      statusCode: 200,
      multiValueHeaders: {
        'Set-Cookie': [
          `abyss_pending=${pending}; Path=/; Max-Age=${PENDING_MAX_AGE}; HttpOnly; Secure; SameSite=Lax`,
        ],
      },
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ result }),
    };
  }

  // ---- action: declare ----
  if (body.action === 'declare') {
    const cookies = parseCookies(event.headers.cookie || event.headers.Cookie);
    const pendingPayload = verify(cookies.abyss_pending, secret);
    const freshEnough = pendingPayload && (Date.now() - pendingPayload.t) < PENDING_MAX_AGE * 1000;

    if (!pendingPayload || pendingPayload.r !== 'ELIGIBLE' || !freshEnough) {
      return { statusCode: 403, body: JSON.stringify({ error: 'No valid eligibility check on record. Run the check again.' }) };
    }
    if (!body.declarationVersion || body.accepted !== true) {
      return { statusCode: 400, body: JSON.stringify({ error: 'Declaration not accepted' }) };
    }

    const gate = sign({ r: 'ELIGIBLE', doc: body.declarationVersion, t: Date.now() }, secret);
    return {
      statusCode: 200,
      multiValueHeaders: {
        'Set-Cookie': [
          `abyss_gate=${gate}; Path=/; Max-Age=${GATE_MAX_AGE}; HttpOnly; Secure; SameSite=Lax`,
          `abyss_pending=; Path=/; Max-Age=0; HttpOnly; Secure; SameSite=Lax`,
        ],
      },
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ result: 'GRANTED' }),
    };
  }

  return { statusCode: 400, body: JSON.stringify({ error: 'Unknown action' }) };
};
