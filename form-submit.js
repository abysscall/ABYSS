/**
 * Submit HTML forms to Netlify Forms (production) with a clear local-dev fallback.
 */
(function (global) {
  function isLocalHost() {
    const host = global.location.hostname;
    return host === 'localhost' || host === '127.0.0.1' || host === '';
  }

  async function submitNetlifyForm(formName, fields) {
    const body = new URLSearchParams({ 'form-name': formName });
    for (const [key, value] of Object.entries(fields)) {
      body.append(key, value == null ? '' : String(value));
    }

    const response = await fetch('/', {
      method: 'POST',
      headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
      body: body.toString(),
    });

    if (!response.ok) {
      throw new Error('Form submission failed (' + response.status + ')');
    }
  }

  global.AbyssForms = {
    isLocalHost,
    submitNetlifyForm,
  };
})(typeof window !== 'undefined' ? window : globalThis);
