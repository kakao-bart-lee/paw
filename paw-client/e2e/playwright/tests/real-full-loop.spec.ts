import { expect, test } from '@playwright/test';

import {
  apiBaseUrl,
  fetchConversations,
  fetchMessages,
  gotoChatWithBootstrapTokens,
  insertIncomingMessage,
  loginViaRealOtp,
  seedConversationForPhone,
  startFailureCollection,
} from './real-helpers';

test.describe('real server full loop', () => {
  test.setTimeout(180_000);

  test('login -> conversations -> send/receive -> in-session restore -> browser refresh policy', async ({
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
    await expect
      .poll(async () => {
        const response = await fetchConversations(auth.accessToken);
        return response.conversations.some(
          (conversation) =>
            conversation.id === seeded.conversationId &&
            conversation.name === seeded.title,
        );
      })
      .toBe(true);

    await page.goto('/#/profile/me', { waitUntil: 'networkidle' });
    await expect(page).toHaveURL(/#\/profile\/me$/);
    await page.goto('/#/chat', { waitUntil: 'networkidle' });
    await expect(page).toHaveURL(/#\/chat$/);

    await page.goto(`/#/chat/${seeded.conversationId}`, { waitUntil: 'networkidle' });
    const myMessage = `hello-${Date.now()}`;
    const sendResponse = page.waitForResponse(
      (response) =>
        response.url() ===
          `${apiBaseUrl}/conversations/${seeded.conversationId}/messages` &&
        response.request().method() === 'POST' &&
        response.status() === 200,
    );
    await page.locator('input, textarea').last().fill(myMessage);
    await page.getByRole('button', { name: '전송' }).click();
    await sendResponse;
    await expect
      .poll(async () => {
        const response = await fetchMessages(auth.accessToken, seeded.conversationId);
        return response.messages.some((message) => message.content === myMessage);
      })
      .toBe(true);

    const incoming = `incoming-${Date.now()}`;
    await insertIncomingMessage({
      conversationId: seeded.conversationId,
      senderId: seeded.peerId,
      content: incoming,
    });
    await expect
      .poll(async () => {
        const response = await fetchMessages(auth.accessToken, seeded.conversationId);
        return response.messages.some((message) => message.content === incoming);
      }, { timeout: 10_000 })
      .toBe(true);

    // In-session restore: navigate away/back and verify message history is still shown.
    await page.goto('/#/settings', { waitUntil: 'networkidle' });
    await page.goto('/#/chat', { waitUntil: 'networkidle' });
    await page.goto(`/#/chat/${seeded.conversationId}`, { waitUntil: 'networkidle' });
    await expect(page).toHaveURL(new RegExp(`#\\/chat\\/${seeded.conversationId}$`));

    // Web policy: full browser refresh does not auto-restore auth session.
    await page.reload({ waitUntil: 'networkidle' });
    await expect(page).toHaveURL(/#\/(login|auth\/phone)$/);

    expect(failures, JSON.stringify(failures, null, 2)).toEqual([]);
  });
});
