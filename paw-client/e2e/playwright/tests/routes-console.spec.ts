import { expect, test } from '@playwright/test';

type ConsoleFailure = {
  kind: 'console' | 'pageerror';
  message: string;
};

function startFailureCollection(page: import('@playwright/test').Page) {
  const failures: ConsoleFailure[] = [];

  page.on('console', (msg) => {
    if (msg.type() === 'error') {
      failures.push({ kind: 'console', message: msg.text() });
    }
  });

  page.on('pageerror', (err) => {
    failures.push({ kind: 'pageerror', message: err.message });
  });

  return failures;
}

async function gotoFlutterRoute(
  page: import('@playwright/test').Page,
  routePath: string,
) {
  await page.goto('/#' + routePath, { waitUntil: 'networkidle' });
}

test.describe('web auth guard + console stability', () => {
  test('unauthenticated navigation to /login, /chat, /profile/me keeps console errors at 0', async ({
    page,
  }) => {
    const failures = startFailureCollection(page);

    await gotoFlutterRoute(page, '/login');
    await page.waitForTimeout(500);

    await gotoFlutterRoute(page, '/chat');
    await page.waitForTimeout(500);

    await gotoFlutterRoute(page, '/profile/me');
    await page.waitForTimeout(500);

    await page.reload({ waitUntil: 'networkidle' });
    await page.waitForTimeout(500);

    expect(failures, JSON.stringify(failures, null, 2)).toEqual([]);
  });
});
