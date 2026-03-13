import type { Page } from '@playwright/test';
import { Client } from 'pg';
import { randomUUID } from 'crypto';

export type ConsoleFailure = {
  kind: 'console' | 'pageerror';
  message: string;
};

export const apiBaseUrl =
  process.env.PAW_API_BASE_URL ?? 'http://127.0.0.1:38173';
export const webBaseUrl =
  process.env.PAW_WEB_BASE_URL ?? 'http://127.0.0.1:38481';
export const dbUrl =
  process.env.DATABASE_URL ??
  'postgres://postgres:postgres@127.0.0.1:35432/paw';

export function startFailureCollection(page: Page) {
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

export async function enableFlutterAccessibility(page: Page) {
  const toggle = page.getByRole('button', { name: 'Enable accessibility' });
  if (await toggle.isVisible().catch(() => false)) {
    await toggle.click();
  }
}

export async function gotoChatWithBootstrapTokens(
  page: Page,
  tokens: { accessToken: string; refreshToken: string },
) {
  const url = new URL(webBaseUrl);
  url.hash =
    `#/chat?e2e_access_token=${encodeURIComponent(tokens.accessToken)}` +
    `&e2e_refresh_token=${encodeURIComponent(tokens.refreshToken)}`;

  await page.goto(url.toString(), { waitUntil: 'networkidle' });
  await page.waitForURL(/#\/chat/);
  await enableFlutterAccessibility(page);
  await page.evaluate(() => {
    window.history.replaceState({}, '', '/#/chat');
  });
}

export function uniquePhone() {
  const suffix = `${Date.now()}${Math.floor(Math.random() * 10000)}`
    .replace(/\D/g, '')
    .slice(-8);
  return `+8210${suffix}`;
}

export async function requestOtp(phone: string) {
  const response = await fetch(`${apiBaseUrl}/auth/request-otp`, {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify({ phone }),
  });

  if (!response.ok) {
    const text = await response.text();
    throw new Error(`requestOtp failed: ${response.status} ${text}`);
  }
}

export async function fetchConversations(accessToken: string) {
  const response = await fetch(`${apiBaseUrl}/conversations`, {
    headers: { authorization: `Bearer ${accessToken}` },
  });

  if (!response.ok) {
    const text = await response.text();
    throw new Error(`fetchConversations failed: ${response.status} ${text}`);
  }

  return (await response.json()) as {
    conversations: Array<{
      id: string;
      name: string;
      last_message: string | null;
      unread_count: number;
    }>;
  };
}

export async function verifyOtp(phone: string, code: string) {
  const response = await fetch(`${apiBaseUrl}/auth/verify-otp`, {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify({ phone, code }),
  });

  if (!response.ok) {
    const text = await response.text();
    throw new Error(`verifyOtp failed: ${response.status} ${text}`);
  }

  return (await response.json()) as { session_token: string };
}

export async function registerDevice(sessionToken: string, deviceName: string) {
  const response = await fetch(`${apiBaseUrl}/auth/register-device`, {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify({
      session_token: sessionToken,
      device_name: deviceName,
      ed25519_public_key: 'AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=',
    }),
  });

  if (!response.ok) {
    const text = await response.text();
    throw new Error(`registerDevice failed: ${response.status} ${text}`);
  }

  return (await response.json()) as {
    access_token: string;
    refresh_token: string;
  };
}

export async function fetchLatestOtp(phone: string) {
  const client = new Client({ connectionString: dbUrl });
  await client.connect();

  try {
    for (let i = 0; i < 20; i++) {
      const result = await client.query<{ code: string }>(
        `SELECT code
         FROM otp_codes
         WHERE phone = $1
         ORDER BY created_at DESC
         LIMIT 1`,
        [phone],
      );

      const code = result.rows[0]?.code;
      if (code) {
        return code;
      }
      await new Promise((resolve) => setTimeout(resolve, 300));
    }
  } finally {
    await client.end();
  }

  throw new Error('OTP not found in database');
}

export async function seedConversationForPhone(phone: string) {
  const client = new Client({ connectionString: dbUrl });
  await client.connect();

  try {
    const userResult = await client.query<{ id: string }>(
      `SELECT id FROM users WHERE phone = $1 LIMIT 1`,
      [phone],
    );
    const userId = userResult.rows[0]?.id;
    if (!userId) {
      throw new Error(`user not found for phone=${phone}`);
    }

    const peerPhone = `+821099${`${Date.now()}`.slice(-6)}`;
    const peerResult = await client.query<{ id: string }>(
      `INSERT INTO users (phone, display_name)
       VALUES ($1, 'e2e-peer')
       ON CONFLICT (phone) DO UPDATE SET updated_at = NOW()
       RETURNING id`,
      [peerPhone],
    );
    const peerId = peerResult.rows[0].id;

    const conversationId = randomUUID();
    const title = `e2e-${Date.now()}`;

    await client.query(
      `INSERT INTO conversations (id, type, title, created_by, created_at, updated_at, last_message_at)
       VALUES ($1, 'direct', $2, $3, NOW(), NOW(), NOW())`,
      [conversationId, title, userId],
    );

    await client.query(
      `INSERT INTO conversation_members (conversation_id, user_id, role)
       VALUES ($1, $2, 'owner'), ($1, $3, 'member')`,
      [conversationId, userId, peerId],
    );

    await client.query(
      `INSERT INTO conversation_seq (conversation_id, last_seq)
       VALUES ($1, 0)
       ON CONFLICT (conversation_id) DO NOTHING`,
      [conversationId],
    );

    return { conversationId, title, peerId };
  } finally {
    await client.end();
  }
}

export async function insertIncomingMessage(params: {
  conversationId: string;
  senderId: string;
  content: string;
}) {
  const client = new Client({ connectionString: dbUrl });
  await client.connect();

  try {
    const seqResult = await client.query<{ next: string }>(
      `SELECT next_message_seq($1) as next`,
      [params.conversationId],
    );
    const seq = Number(seqResult.rows[0].next);

    await client.query(
      `INSERT INTO messages (id, conversation_id, sender_id, seq, content, format, blocks)
       VALUES ($1, $2, $3, $4, $5, 'plain', '[]'::jsonb)`,
      [randomUUID(), params.conversationId, params.senderId, seq, params.content],
    );
  } finally {
    await client.end();
  }
}

export async function fetchMessages(accessToken: string, conversationId: string) {
  const response = await fetch(
    `${apiBaseUrl}/conversations/${conversationId}/messages?limit=50`,
    {
      headers: { authorization: `Bearer ${accessToken}` },
    },
  );

  if (!response.ok) {
    const text = await response.text();
    throw new Error(`fetchMessages failed: ${response.status} ${text}`);
  }

  return (await response.json()) as {
    messages: Array<{
      content: string;
    }>;
  };
}

export async function loginViaRealOtp(page: Page, phone = uniquePhone()) {
  await requestOtp(phone);
  const otp = await fetchLatestOtp(phone);
  const verify = await verifyOtp(phone, otp);
  const tokens = await registerDevice(verify.session_token, 'e2e-device');

  await gotoChatWithBootstrapTokens(page, {
    accessToken: tokens.access_token,
    refreshToken: tokens.refresh_token,
  });

  return {
    phone,
    accessToken: tokens.access_token,
    refreshToken: tokens.refresh_token,
  };
}
