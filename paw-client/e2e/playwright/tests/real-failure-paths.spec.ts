import { expect, test } from '@playwright/test';

import {
  apiBaseUrl,
  gotoChatWithBootstrapTokens,
  loginViaRealOtp,
  startFailureCollection,
} from './real-helpers';

test.describe('real server failure paths', () => {
  test.setTimeout(180_000);

  test('401 on protected bootstrap request clears session and returns to explicit login flow', async ({
    page,
  }) => {
    const failures = startFailureCollection(page);

    const auth = await loginViaRealOtp(page);

    await page.route('**/users/me*', async (route) => {
      await route.fulfill({
        status: 401,
        contentType: 'application/json',
        body: JSON.stringify({
          error: 'invalid_token',
          message: 'Access token is invalid',
          request_id: 'pw-e2e-401',
        }),
      });
    });

    await gotoChatWithBootstrapTokens(page, {
      accessToken: auth.accessToken,
      refreshToken: auth.refreshToken,
    });
    await page.goto('/#/profile/me', { waitUntil: 'domcontentloaded' });
    await expect
      .poll(async () => page.url(), { timeout: 10_000 })
      .toMatch(/#\/(login|auth\/phone)$/);

    expect(failures, JSON.stringify(failures, null, 2)).toEqual([]);
  });

  test('503 on conversations request shows error UI without console failures', async ({
    page,
  }) => {
    const failures = startFailureCollection(page);

    const auth = await loginViaRealOtp(page);

    await page.route('**/conversations*', async (route) => {
      await route.fulfill({
        status: 503,
        contentType: 'application/json',
        body: JSON.stringify({
          error: 'server',
          message: 'temporarily unavailable',
        }),
      });
    });

    await gotoChatWithBootstrapTokens(page, {
      accessToken: auth.accessToken,
      refreshToken: auth.refreshToken,
    });
    await expect(
      page.getByText(/temporarily unavailable|서버에 일시적인 문제가 있습니다/),
    ).toBeVisible();

    expect(
      failures.filter(
        (failure) =>
          !(
            failure.kind === 'pageerror' &&
            failure.message.trim() === 'Error'
          ),
        ),
      JSON.stringify(failures, null, 2),
    ).toEqual([]);
  });
});
