// netlify/edge-functions/gate.js
//
// Enforces the ABYSS Gateway (ADR-0025) at the network edge, before any
// protected static page is served. This is the actual enforcement layer --
// client-side JavaScript on index.html/invest.html/wallet.html makes no
// access decision; it never runs unless this function already let the
// request through.
//
// Required environment variables (Netlify dashboard -> Site settings ->
// Environment variables):
//   ABYSS_GATE_SECRET  -- must be IDENTICAL to the one used by
//                         netlify/functions/eligibility.js
//   ABYSS_ADMIN_KEY    -- a separate secret only you know, for the
//                         developer/admin bypass. Optional -- the gate
//                         still works without it, you just won't have
//                         a bypass.

export default async (request, context) => {
  const url = new URL(request.url);

  const adminKey = Deno.env.get('ABYSS_ADMIN_KEY');
  const providedAdmin = url.searchParams.get('admin');
  const cookieHeader = request.headers.get('cookie') || '';

  // Admin/developer bypass. This is a convenience, not real access control:
  // anyone who obtains this URL parameter can also use it. It exists so the
  // site owner can preview pages without repeating the gate flow -- it does
  // not protect anything sensitive. The one genuinely sensitive step
  // (accepting investor funds) stays fully gated elsewhere regardless.
  if (adminKey && providedAdmin === adminKey) {
    const response = await context.next();
    response.headers.append(
      'Set-Cookie',
      'abyss_admin=1; Path=/; Max-Age=2592000; HttpOnly; Secure; SameSite=Lax'
    );
    return response;
  }
  if (cookieHeader.includes('abyss_admin=1')) {
    return context.next();
  }

  const secret = Deno.env.get('ABYSS_GATE_SECRET');
  const gateCookie = parseCookie(cookieHeader, 'abyss_gate');

  if (secret && gateCookie && (await verify(gateCookie, secret))) {
    return context.next();
  }

  return Response.redirect(new URL('/gateway.html', request.url), 307);
};

function parseCookie(header, name) {
  const match = header.match(new RegExp('(?:^|;\\s*)' + name + '=([^;]*)'));
  return match ? decodeURIComponent(match[1]) : null;
}

async function verify(token, secret) {
  if (!token || !token.includes('.')) return false;
  const [payload, sig] = token.split('.');
  const expected = await hmac(payload, secret);
  return expected === sig;
}

async function hmac(message, secret) {
  const enc = new TextEncoder();
  const key = await crypto.subtle.importKey(
    'raw', enc.encode(secret), { name: 'HMAC', hash: 'SHA-256' }, false, ['sign']
  );
  const sigBuf = await crypto.subtle.sign('HMAC', key, enc.encode(message));
  return base64url(new Uint8Array(sigBuf));
}

function base64url(bytes) {
  let bin = '';
  bytes.forEach((b) => (bin += String.fromCharCode(b)));
  return btoa(bin).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');
}

export const config = {
  path: ['/', '/index.html', '/invest.html', '/wallet.html'],
};
