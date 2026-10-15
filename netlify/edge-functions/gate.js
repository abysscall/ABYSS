// Enforces the ABYSS Gateway (ADR-0025) before a protected page is served.
//
// Environment variables (set in the Netlify UI, with Functions/Edge scope):
//   ABYSS_GATE_SECRET   signs visitor tokens; it must match eligibility.js.
//   ABYSS_ADMIN_SECRET  signs admin tokens; defaults to ABYSS_GATE_SECRET.
//   ABYSS_ADMIN_KEY     optional shared secret for the owner bypass.

const TOKEN_VERSION = 1;
const ADMIN_TTL_MS = 1000 * 60 * 60 * 24 * 30;
const CLOCK_SKEW_MS = 1000 * 60;
const MAX_TOKEN_CHARS = 4096;
const HMAC_SHA256_B64URL_LENGTH = 43;
const ADMIN_CHALLENGE = 'abyss-admin-bypass-v1';

export default async function gate(request, context) {
  const url = new URL(request.url);
  const gateSecret = Netlify.env.get('ABYSS_GATE_SECRET');
  const adminSecret = Netlify.env.get('ABYSS_ADMIN_SECRET') || gateSecret;
  const adminKey = Netlify.env.get('ABYSS_ADMIN_KEY');

  if (!gateSecret) {
    return new Response('Gateway misconfigured', {
      status: 503,
      headers: { 'Cache-Control': 'no-store' },
    });
  }

  const cookieHeader = request.headers.get('cookie') || '';
  const providedAdmin = request.headers.get('x-abyss-admin') || url.searchParams.get('admin');

  if (adminKey && providedAdmin && await adminKeyMatches(providedAdmin, adminKey)) {
    const token = await signToken({ t: 'admin' }, adminSecret, ADMIN_TTL_MS);

    // A query parameter is useful for a one-time owner link, but must never
    // remain in browser history, Referer headers, or copied URLs.
    if (url.searchParams.has('admin')) {
      const cleanUrl = new URL(url);
      cleanUrl.searchParams.delete('admin');
      return redirectWithCookie(cleanUrl, 'abyss_admin', token, ADMIN_TTL_MS / 1000);
    }

    return nextWithSecurityHeaders(context, {
      name: 'abyss_admin',
      value: token,
      maxAgeSec: ADMIN_TTL_MS / 1000,
    });
  }

  const adminCookie = parseCookie(cookieHeader, 'abyss_admin');
  if (adminCookie && await verifyToken(adminCookie, adminSecret, 'admin')) {
    return nextWithSecurityHeaders(context);
  }

  const gateCookie = parseCookie(cookieHeader, 'abyss_gate');
  if (gateCookie && await verifyToken(gateCookie, gateSecret, 'gate')) {
    return nextWithSecurityHeaders(context);
  }

  const redirect = new URL('/gateway.html', request.url);
  if (url.pathname !== '/' && url.pathname !== '/index.html') {
    // Never forward an attempted admin key to the public gateway page.
    const returnUrl = new URL(url);
    returnUrl.searchParams.delete('admin');
    redirect.searchParams.set('next', returnUrl.pathname + returnUrl.search);
  }
  return Response.redirect(redirect, 307);
}

async function nextWithSecurityHeaders(context, cookie) {
  const upstream = await context.next();

  // Responses returned by context.next() can have immutable Headers in edge
  // runtimes. Reconstructing the Response makes the copied headers mutable.
  const response = new Response(upstream.body, upstream);
  if (cookie) appendCookie(response, cookie.name, cookie.value, cookie.maxAgeSec);
  hardenResponse(response);
  return response;
}

function redirectWithCookie(location, name, value, maxAgeSec) {
  const response = new Response(null, {
    status: 303,
    headers: {
      Location: location.toString(),
      'Cache-Control': 'no-store',
      'X-Robots-Tag': 'noindex, nofollow',
    },
  });
  appendCookie(response, name, value, maxAgeSec);
  return response;
}

function parseCookie(header, name) {
  for (const raw of header.split(';')) {
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

function appendCookie(response, name, value, maxAgeSec) {
  response.headers.append(
    'Set-Cookie',
    name + '=' + encodeURIComponent(value) + '; Path=/; Max-Age=' + Math.floor(maxAgeSec)
      + '; HttpOnly; Secure; SameSite=Lax',
  );
}

function hardenResponse(response) {
  response.headers.set('Cache-Control', 'private, no-store');
  response.headers.set('X-Robots-Tag', 'noindex, nofollow');
}

async function adminKeyMatches(provided, expected) {
  if (typeof provided !== 'string' || typeof expected !== 'string' || provided.length > 1024) {
    return false;
  }

  // Verify a fixed challenge with the supplied value as an HMAC key. This
  // delegates the comparison to Web Crypto rather than a JavaScript loop.
  const expectedProof = await hmacSign(ADMIN_CHALLENGE, expected);
  return hmacVerify(ADMIN_CHALLENGE, expectedProof, provided);
}

async function signToken(claims, secret, ttlMs) {
  const now = Date.now();
  const payload = { ...claims, iat: now, exp: now + ttlMs, v: TOKEN_VERSION };
  const payloadB64 = base64url(new TextEncoder().encode(JSON.stringify(payload)));
  const signature = await hmacSign(payloadB64, secret);
  return payloadB64 + '.' + signature;
}

async function verifyToken(token, secret, expectedType) {
  if (typeof token !== 'string' || token.length > MAX_TOKEN_CHARS) return null;

  const dot = token.indexOf('.');
  if (dot <= 0 || dot !== token.lastIndexOf('.')) return null;

  const payloadB64 = token.slice(0, dot);
  const signatureB64 = token.slice(dot + 1);
  if (!payloadB64 || signatureB64.length !== HMAC_SHA256_B64URL_LENGTH) return null;
  if (!await hmacVerify(payloadB64, signatureB64, secret)) return null;

  let claims;
  try {
    claims = JSON.parse(new TextDecoder('utf-8', { fatal: true }).decode(base64urlDecode(payloadB64)));
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

async function hmacSign(message, secret) {
  const encoder = new TextEncoder();
  const key = await crypto.subtle.importKey(
    'raw', encoder.encode(secret), { name: 'HMAC', hash: 'SHA-256' }, false, ['sign'],
  );
  const signature = await crypto.subtle.sign('HMAC', key, encoder.encode(message));
  return base64url(new Uint8Array(signature));
}

async function hmacVerify(message, signatureB64, secret) {
  let signature;
  try {
    signature = base64urlDecode(signatureB64);
  } catch {
    return false;
  }

  try {
    const encoder = new TextEncoder();
    const key = await crypto.subtle.importKey(
      'raw', encoder.encode(secret), { name: 'HMAC', hash: 'SHA-256' }, false, ['verify'],
    );
    return await crypto.subtle.verify('HMAC', key, signature, encoder.encode(message));
  } catch {
    return false;
  }
}

const B64_ALPHABET = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_';
const B64_LOOKUP = (() => {
  const lookup = new Int16Array(256).fill(-1);
  for (let i = 0; i < B64_ALPHABET.length; i += 1) lookup[B64_ALPHABET.charCodeAt(i)] = i;
  return lookup;
})();

function base64url(bytes) {
  let output = '';
  let index = 0;
  for (; index + 2 < bytes.length; index += 3) {
    const value = (bytes[index] << 16) | (bytes[index + 1] << 8) | bytes[index + 2];
    output += B64_ALPHABET[(value >>> 18) & 63]
      + B64_ALPHABET[(value >>> 12) & 63]
      + B64_ALPHABET[(value >>> 6) & 63]
      + B64_ALPHABET[value & 63];
  }
  if (bytes.length - index === 1) {
    const value = bytes[index] << 16;
    output += B64_ALPHABET[(value >>> 18) & 63] + B64_ALPHABET[(value >>> 12) & 63];
  } else if (bytes.length - index === 2) {
    const value = (bytes[index] << 16) | (bytes[index + 1] << 8);
    output += B64_ALPHABET[(value >>> 18) & 63]
      + B64_ALPHABET[(value >>> 12) & 63]
      + B64_ALPHABET[(value >>> 6) & 63];
  }
  return output;
}

function base64urlDecode(input) {
  if (typeof input !== 'string' || input.length % 4 === 1) throw new TypeError('Invalid base64url');

  const output = new Uint8Array(Math.floor((input.length * 6) / 8));
  let accumulator = 0;
  let bits = 0;
  let outputIndex = 0;

  for (let index = 0; index < input.length; index += 1) {
    const code = input.charCodeAt(index);
    if (code > 255 || B64_LOOKUP[code] < 0) throw new TypeError('Invalid base64url');
    accumulator = (accumulator << 6) | B64_LOOKUP[code];
    bits += 6;
    if (bits >= 8) {
      bits -= 8;
      output[outputIndex] = (accumulator >>> bits) & 0xff;
      outputIndex += 1;
      accumulator &= (1 << bits) - 1;
    }
  }

  return output;
}

export const config = {
  path: ['/', '/index.html', '/invest', '/invest.html', '/wallet', '/wallet.html'],
};
