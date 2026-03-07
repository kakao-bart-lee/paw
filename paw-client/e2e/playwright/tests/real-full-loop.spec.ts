import { expect, test } from '@playwright/test';
import { Client } from 'pg';
import { randomUUID } from 'crypto';

type ConsoleFailure = {
  kind: 'console' | 'pageerror';
  message: string;
};

const apiBaseUrl = process.env.PAW_API_BASE_URL ?? 'http://127.0.0.1:3000';
const dbUrl =
  process.env.DATABASE_URL ??
  'postgres://postgres:postgres@127.0.0.1:5432/paw';

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

function uniquePhone() {
  const suffix = `${Date.now()}`.slice(-8);
  return `+8210${suffix}`;
}

function localPhoneDigits(phone: string) {
  return phone.replace('+82', '').replace(/\D/g, '');
}

async function requestOtp(phone: string) {
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

async function fetchLatestOtp(phone: string) {
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

async function seedConversationForPhone(phone: string) {
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

async function insertIncomingMessage(params: {
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

test.describe('real server full loop', () => {
  test.setTimeout(180_000);

  test('login -> conversations -> send/receive -> in-session restore -> browser refresh policy', async ({
    page,
  }) => {
    const failures = startFailureCollection(page);

    const phone = uniquePhone();
    await page.goto('/#/auth/phone', { waitUntil: 'networkidle' });
    await page.waitForTimeout(1500);

    const input = page.locator('input').first();
    await input.waitFor({ state: 'visible', timeout: 90_000 });
    await input.fill(localPhoneDigits(phone));
    await Promise.all([
      requestOtp(phone),
      page.getByRole('button', { name: '인증번호 받기' }).click(),
    ]);

    await expect(page.getByText('인증번호 입력')).toBeVisible();

    const otp = await fetchLatestOtp(phone);
    await page.locator('input').first().fill(otp);
    await page.getByRole('button', { name: '확인' }).click();

    await expect(page.getByText('이 기기 이름 설정')).toBeVisible();

    const seeded = await seedConversationForPhone(phone);

    await page.locator('input').first().fill('e2e-device');
    await page.getByRole('button', { name: '시작하기' }).click();

    await expect(page).toHaveURL(/#\/chat/);
    await expect(page.getByText(seeded.title)).toBeVisible();

    await page.getByText(seeded.title).first().click();
    const myMessage = `hello-${Date.now()}`;
    await page.locator('input, textarea').last().fill(myMessage);
    await page.getByRole('button', { name: '전송' }).click();
    await expect(page.getByText(myMessage)).toBeVisible();

    const incoming = `incoming-${Date.now()}`;
    await insertIncomingMessage({
      conversationId: seeded.conversationId,
      senderId: seeded.peerId,
      content: incoming,
    });
    await expect(page.getByText(incoming)).toBeVisible({ timeout: 10_000 });

    // In-session restore: navigate away/back and verify message history is still shown.
    await page.goto('/#/settings', { waitUntil: 'networkidle' });
    await page.goto(`/#/chat/${seeded.conversationId}`, { waitUntil: 'networkidle' });
    await expect(page.getByText(myMessage)).toBeVisible();

    // Web policy: full browser refresh does not auto-restore auth session.
    await page.reload({ waitUntil: 'networkidle' });
    await expect(page).toHaveURL(/#\/(login|auth\/phone)$/);

    expect(failures, JSON.stringify(failures, null, 2)).toEqual([]);
  });
});
