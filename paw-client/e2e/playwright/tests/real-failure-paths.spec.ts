import { expect, test } from '@playwright/test';

import {
  apiBaseUrl,
  enableFlutterAccessibility,
  loginViaRealOtp,
  startFailureCollection,
} from './real-helpers';

test.describe('real server failure paths', () => {
  test.setTimeout(180_000);

  test('401 on profile request clears session and returns to explicit login flow', async ({
    page,
  }) => {
    const failures = startFailureCollection(page);

    await loginViaRealOtp(page);

    await page.route(`${apiBaseUrl}/users/me`, async (route) => {
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

    const unauthorizedResponse = page.waitForResponse(
      (response) =>
        response.url() === `${apiBaseUrl}/users/me` && response.status() == 401,
    );
    await page.goto('/#/profile/me', { waitUntil: 'networkidle' });
    await unauthorizedResponse;
    await expect(page).toHaveURL(/#\/auth\/phone$/);

    expect(failures, JSON.stringify(failures, null, 2)).toEqual([]);
  });

  test('503 on conversations request shows error UI without console failures', async ({
    page,
  }) => {
    const failures = startFailureCollection(page);

    await loginViaRealOtp(page);

    await page.route(`${apiBaseUrl}/conversations`, async (route) => {
      await route.fulfill({
        status: 503,
        contentType: 'application/json',
        body: JSON.stringify({
          error: 'server',
          message: 'temporarily unavailable',
        }),
      });
    });

    const unavailableResponse = page.waitForResponse(
      (response) =>
        response.url() === `${apiBaseUrl}/conversations` &&
        response.status() === 503,
    );
    await page.goto('/#/chat', { waitUntil: 'networkidle' });
    await unavailableResponse;

    expect(failures, JSON.stringify(failures, null, 2)).toEqual([]);
  });
});
