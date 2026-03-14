import { expect, test } from '@playwright/test';

import {
  fetchConversations,
  gotoChatWithBootstrapTokens,
  loginViaRealOtp,
  seedConversationForPhone,
  startFailureCollection,
} from './real-helpers';

test.describe('real server full loop', () => {
  test.setTimeout(180_000);

  test('login -> conversations -> send -> browser refresh policy', async ({
    page,
  }) => {
    const failures = startFailureCollection(page);
    const auth = await loginViaRealOtp(page);
    const seeded = await seedConversationForPhone(auth.phone);

    await gotoChatWithBootstrapTokens(page, {
      accessToken: auth.accessToken,
      refreshToken: auth.refreshToken,
    });

    await expect(page).toHaveURL(/#\/chat/);
    await expect(page.getByText('대화를 선택하세요')).toBeVisible();

    const conversations = await fetchConversations(auth.accessToken);
    const firstConversationId =
      conversations.conversations.find((item) => item.id === seeded.conversationId)?.id ??
      conversations.conversations[0]?.id;
    expect(firstConversationId).toBeTruthy();
    await page.goto(`/#/chat/${firstConversationId}`, {
      waitUntil: 'domcontentloaded',
    });

    await page.goto('/#/profile/me', { waitUntil: 'domcontentloaded' });
    await expect(page).toHaveURL(/#\/profile\/me$/);
    await page.goto('/#/chat', { waitUntil: 'domcontentloaded' });
    await expect(page).toHaveURL(/#\/chat$/);
    await page.goto(`/#/chat/${firstConversationId}`, {
      waitUntil: 'domcontentloaded',
    });

    const myMessage = `hello-${Date.now()}`;
    await page.locator('input, textarea').last().fill(myMessage);
    await page.getByRole('button', { name: '전송' }).click();
    await page.goto('/#/settings', { waitUntil: 'domcontentloaded' });
    await page.goto('/#/chat', { waitUntil: 'domcontentloaded' });

    // Web policy: full browser refresh does not auto-restore auth session.
    await page.reload({ waitUntil: 'domcontentloaded' });
    await expect(page).toHaveURL(/#\/(login|auth\/phone)$/);

    expect(failures, JSON.stringify(failures, null, 2)).toEqual([]);
  });
});
