// Issues the short-lived pending token and the ABYSS Gateway token.
// The token format deliberately matches netlify/edge-functions/gate.js:
// base64url(JSON claims) + '.' + base64url(HMAC-SHA256(payload)).

const crypto = require('crypto');

// Placeholder only. Counsel must supply feature-specific production policy.
const RESTRICTED_CODES = new Set(['420', '49', '33']);
const DECLARATION_VERSION = 'ETD-001-1.0';
const TOKEN_VERSION = 1;
const PENDING_TTL_MS = 1000 * 60 * 10;
const GATE_TTL_MS = 1000 * 60 * 60 * 24 * 7;
const CLOCK_SKEW_MS = 1000 * 60;
const MAX_TOKEN_CHARS = 4096;
const HMAC_SHA256_BYTES = 32;

const JSON_HEADERS = {
  'Content-Type': 'application/json; charset=utf-8',
  'Cache-Control': 'no-store',
  'X-Content-Type-Options': 'nosniff',
};

function base64url(buffer) {
  return buffer.toString('base64').replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');
}

function base64urlDecode(value) {
  if (typeof value !== 'string' || !/^[A-Za-z0-9_-]*$/.test(value) || value.length % 4 === 1) {
    throw new TypeError('Invalid base64url');
  }
  const padding = '='.repeat((4 - (value.length % 4)) % 4);
  const decoded = Buffer.from(value + padding, 'base64');
  if (base64url(decoded) !== value) throw new TypeError('Non-canonical base64url');
  return decoded;
}

function signToken(claims, secret, ttlMs) {
  const now = Date.now();
  const payload = { ...claims, iat: now, exp: now + ttlMs, v: TOKEN_VERSION };
  const payloadB64 = base64url(Buffer.from(JSON.stringify(payload), 'utf8'));
  const signature = base64url(crypto.createHmac('sha256', secret).update(payloadB64, 'utf8').digest());
  return payloadB64 + '.' + signature;
}

function verifyToken(token, secret, expectedType) {
  if (typeof token !== 'string' || token.length > MAX_TOKEN_CHARS) return null;

  const dot = token.indexOf('.');
  if (dot <= 0 || dot !== token.lastIndexOf('.')) return null;
  const payloadB64 = token.slice(0, dot);
  const signatureB64 = token.slice(dot + 1);

  let suppliedSignature;
  try {
    suppliedSignature = base64urlDecode(signatureB64);
  } catch {
    return null;
  }
  if (suppliedSignature.length !== HMAC_SHA256_BYTES) return null;

  const expectedSignature = crypto.createHmac('sha256', secret).update(payloadB64, 'utf8').digest();
  if (!crypto.timingSafeEqual(expectedSignature, suppliedSignature)) return null;

  let claims;
  try {
    claims = JSON.parse(base64urlDecode(payloadB64).toString('utf8'));
  } catch {
    return null;
  }
  if (!claims || typeof claims !== 'object') return null;
  if (claims.t !== expectedType || claims.v !== TOKEN_VERSION) return null;
  if (!Number.isSafeInteger(claims.iat) || !Number.isSafeInteger(claims.exp)) return null;

  const now = Date.now();
  if (claims.iat > now + CLOCK_SKEW_MS || claims.exp < claims.iat) return null;
  if (now > claims.exp + CLOCK_SKEW_MS) return null;
  return claims;
}

function parseCookie(header, name) {
  for (const raw of String(header || '').split(';')) {
    const equalAt = raw.indexOf('=');
    if (equalAt === -1 || raw.slice(0, equalAt).trim() !== name) continue;
    try {
      return decodeURIComponent(raw.slice(equalAt + 1).trim());
    } catch {
      return null;
    }
  }
  return null;
}

function json(statusCode, payload, cookies = []) {
  const response = {
    statusCode,
    headers: JSON_HEADERS,
    body: JSON.stringify(payload),
  };
  if (cookies.length > 0) response.multiValueHeaders = { 'Set-Cookie': cookies };
  return response;
}

function cookie(name, value, maxAgeSeconds) {
  return name + '=' + encodeURIComponent(value) + '; Path=/; Max-Age=' + Math.floor(maxAgeSeconds)
    + '; HttpOnly; Secure; SameSite=Lax';
}

function normalizeCountryCode(value) {
  if (typeof value !== 'string') return null;
  const match = /^\+([1-9]\d{0,2})$/.exec(value.trim());
  return match ? match[1] : null;
}

exports.handler = async (event) => {
  const secret = process.env.ABYSS_GATE_SECRET;
  if (!secret) return json(503, { error: 'misconfigured' });
  if (event.httpMethod !== 'POST') {
    return {
      ...json(405, { error: 'method_not_allowed' }),
      headers: { ...JSON_HEADERS, Allow: 'POST' },
    };
  }

  let body;
  try {
    body = JSON.parse(event.body || '{}');
  } catch {
    return json(400, { error: 'bad_json' });
  }
  if (!body || typeof body !== 'object' || Array.isArray(body)) return json(400, { error: 'bad_json' });

  if (body.action === 'check') {
    const countryCode = normalizeCountryCode(body.countryCode);
    if (!countryCode) return json(400, { error: 'invalid_country_code' });

    if (RESTRICTED_CODES.has(countryCode)) {
      // Avoid carrying an old eligible pending cookie into a later denied flow.
      return json(200, { result: 'RESTRICTED' }, [cookie('abyss_pending', '', 0)]);
    }

    const pending = signToken({ t: 'pending' }, secret, PENDING_TTL_MS);
    return json(200, { result: 'ELIGIBLE' }, [cookie('abyss_pending', pending, PENDING_TTL_MS / 1000)]);
  }

  if (body.action === 'declare') {
    const headers = event.headers || {};
    const pending = parseCookie(headers.cookie || headers.Cookie, 'abyss_pending');
    if (!verifyToken(pending, secret, 'pending')) return json(403, { error: 'no_valid_check' });
    if (body.accepted !== true || body.declarationVersion !== DECLARATION_VERSION) {
      return json(400, { error: 'declaration_not_accepted' });
    }

    const gate = signToken({ t: 'gate', doc: DECLARATION_VERSION }, secret, GATE_TTL_MS);
    return json(200, { result: 'GRANTED' }, [
      cookie('abyss_gate', gate, GATE_TTL_MS / 1000),
      cookie('abyss_pending', '', 0),
    ]);
  }

  return json(400, { error: 'unknown_action' });
};
