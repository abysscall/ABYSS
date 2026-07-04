/**
 * Client-side presale quote logic aligned with abyss-tokenomics / abyss-node.
 * Loads data/tokenomics.json and validates quotes for the investor sale page.
 */
(function (global) {
  const ROUND_ID = 'sale-to-investors';

  function parseUsdToCents(value) {
    const cleaned = String(value || '')
      .trim()
      .replace(/^\$/, '')
      .replace(/,/g, '');
    if (!cleaned) return null;
    const parts = cleaned.split('.');
    if (parts.length > 2) return null;
    const dollars = Number(parts[0]);
    if (!Number.isFinite(dollars) || dollars < 0) return null;
    let cents = 0;
    if (parts[1] !== undefined) {
      const frac = parts[1].padEnd(2, '0').slice(0, 2);
      if (!/^\d+$/.test(frac)) return null;
      cents = Number(frac);
    }
    return dollars * 100 + cents;
  }

  function formatUsd(cents) {
    return '$' + (cents / 100).toLocaleString('en-US', {
      minimumFractionDigits: 2,
      maximumFractionDigits: 2,
    });
  }

  function formatAc(amountAc) {
    return amountAc.toLocaleString('en-US') + ' AC';
  }

  function findRound(tokenomics, roundId) {
    return (tokenomics.sale_rounds || []).find((round) => round.id === roundId);
  }

  function quotePresale(tokenomics, amountInput, options) {
    const opts = options || {};
    const round = findRound(tokenomics, opts.roundId || ROUND_ID);
    if (!round) {
      return { ok: false, error: 'Sale round not found in tokenomics data.' };
    }

    const contributionCents = parseUsdToCents(amountInput);
    if (contributionCents === null) {
      return { ok: false, error: 'Enter a valid USD amount.' };
    }

    const minCents = (round.minimum_ticket_usd || 0) * 100;
    if (contributionCents < minCents) {
      return {
        ok: false,
        error: 'Minimum contribution is ' + formatUsd(minCents) + '.',
      };
    }

    const tokensAc = Math.floor(
      (contributionCents * 100_000_000) / round.price_usd_cents
    );
    const capAc = Number(round.token_cap_ac);
    if (tokensAc > capAc) {
      return { ok: false, error: 'Amount exceeds round token cap.' };
    }

    if (opts.kycApproved === false) {
      return { ok: false, error: 'KYC approval is required for a binding quote.' };
    }

    if (minCents >= 10_000_000 && !opts.professional) {
      return {
        ok: false,
        error: 'This round requires accredited/professional investor status.',
      };
    }

    return {
      ok: true,
      roundId: round.id,
      roundName: round.name,
      contributionUsdCents: contributionCents,
      priceUsdCents: round.price_usd_cents,
      tokenAmountAc: tokensAc,
      lockupMonths: 0,
      status: 'quote only; do not accept funds without legal review',
    };
  }

  async function loadTokenomics(url) {
    const response = await fetch(url || 'data/tokenomics.json');
    if (!response.ok) {
      throw new Error('Failed to load tokenomics data');
    }
    return response.json();
  }

  global.AbyssPresaleQuote = {
    ROUND_ID,
    parseUsdToCents,
    formatUsd,
    formatAc,
    quotePresale,
    loadTokenomics,
  };
})(typeof window !== 'undefined' ? window : globalThis);
