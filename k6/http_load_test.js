/**
 * Paw HTTP API Load Test — k6
 *
 * Endpoints tested:
 *   POST /auth/request-otp
 *   POST /auth/verify-otp
 *   POST /auth/register-device
 *   GET  /conversations
 *   POST /conversations
 *   POST /conversations/:id/messages
 *   GET  /conversations/:id/messages
 *
 * Run:
 *   k6 run k6/http_load_test.js
 *   k6 run --env BASE_URL=http://localhost:3000 k6/http_load_test.js
 */

import http from "k6/http";
import { check, group, sleep } from "k6";
import { Counter, Rate, Trend } from "k6/metrics";
import { uuidv4 } from "https://jslib.k6.io/k6-utils/1.4.0/index.js";

const BASE_URL = __ENV.BASE_URL || "http://localhost:3000";
const TOKEN = __ENV.TOKEN || "REPLACE_WITH_JWT_ACCESS_TOKEN";
const PHONE_PREFIX = __ENV.PHONE_PREFIX || "+8210";

export const options = {
  scenarios: {
    auth_flow: {
      executor: "constant-vus",
      vus: 20,
      duration: "30s",
      exec: "authFlow",
    },
    conversation_crud: {
      executor: "constant-vus",
      vus: 30,
      duration: "30s",
      exec: "conversationFlow",
      startTime: "5s",
    },
    message_throughput: {
      executor: "constant-vus",
      vus: 50,
      duration: "30s",
      exec: "messageFlow",
      startTime: "10s",
    },
  },
  thresholds: {
    http_req_duration: ["p(95)<200"],
    http_req_failed: ["rate<0.05"],
    auth_request_otp_duration: ["p(95)<300"],
    auth_verify_otp_duration: ["p(95)<300"],
    conversation_list_duration: ["p(95)<200"],
    message_send_duration: ["p(95)<200"],
    message_get_duration: ["p(95)<150"],
  },
};

const authRequestOtpDuration = new Trend("auth_request_otp_duration", true);
const authVerifyOtpDuration = new Trend("auth_verify_otp_duration", true);
const conversationListDuration = new Trend("conversation_list_duration", true);
const messageSendDuration = new Trend("message_send_duration", true);
const messageGetDuration = new Trend("message_get_duration", true);
const requestErrors = new Counter("request_errors");

const jsonHeaders = {
  headers: { "Content-Type": "application/json" },
};

function authJsonHeaders(token) {
  return {
    headers: {
      "Content-Type": "application/json",
      Authorization: `Bearer ${token}`,
    },
  };
}

export function authFlow() {
  const phone = `${PHONE_PREFIX}${String(__VU).padStart(8, "0")}`;

  group("POST /auth/request-otp", () => {
    const res = http.post(
      `${BASE_URL}/auth/request-otp`,
      JSON.stringify({ phone }),
      jsonHeaders
    );
    authRequestOtpDuration.add(res.timings.duration);

    const ok = check(res, {
      "request-otp status 200": (r) => r.status === 200,
      "request-otp body has ok": (r) => {
        try {
          return JSON.parse(r.body).ok === true;
        } catch (_) {
          return false;
        }
      },
    });
    if (!ok) requestErrors.add(1);
  });

  sleep(0.5);

  group("POST /auth/verify-otp (expected: invalid without real OTP)", () => {
    const res = http.post(
      `${BASE_URL}/auth/verify-otp`,
      JSON.stringify({ phone, code: "000000" }),
      jsonHeaders
    );
    authVerifyOtpDuration.add(res.timings.duration);

    check(res, {
      "verify-otp responds (200 or 200 with error json)": (r) =>
        r.status === 200,
    });
  });

  sleep(1);
}

export function conversationFlow() {
  const opts = authJsonHeaders(TOKEN);

  group("GET /conversations", () => {
    const res = http.get(`${BASE_URL}/conversations`, opts);
    conversationListDuration.add(res.timings.duration);

    const ok = check(res, {
      "list conversations status 200 or 401": (r) =>
        r.status === 200 || r.status === 401,
    });
    if (!ok) requestErrors.add(1);
  });

  sleep(0.5);

  group("POST /conversations", () => {
    const memberId = uuidv4();
    const res = http.post(
      `${BASE_URL}/conversations`,
      JSON.stringify({
        member_ids: [memberId],
        name: `k6-bench-${__VU}-${__ITER}`,
      }),
      opts
    );

    check(res, {
      "create conversation status 201 or 401": (r) =>
        r.status === 201 || r.status === 401,
    });
  });

  sleep(1);
}

export function messageFlow() {
  const convId =
    __ENV.CONVERSATION_ID || "00000000-0000-0000-0000-000000000001";
  const opts = authJsonHeaders(TOKEN);

  group("POST /conversations/:id/messages", () => {
    const res = http.post(
      `${BASE_URL}/conversations/${convId}/messages`,
      JSON.stringify({
        content: `k6-load-test-vu${__VU}-iter${__ITER}`,
        format: "plain",
        idempotency_key: uuidv4(),
      }),
      opts
    );
    messageSendDuration.add(res.timings.duration);

    check(res, {
      "send message status 200 or 401 or 403": (r) =>
        r.status === 200 || r.status === 401 || r.status === 403,
    });
  });

  sleep(0.3);

  group("GET /conversations/:id/messages", () => {
    const res = http.get(
      `${BASE_URL}/conversations/${convId}/messages?limit=20`,
      opts
    );
    messageGetDuration.add(res.timings.duration);

    check(res, {
      "get messages status 200 or 401 or 403": (r) =>
        r.status === 200 || r.status === 401 || r.status === 403,
    });
  });

  sleep(0.5);
}

export function handleSummary(data) {
  const metrics = [
    ["auth_request_otp_duration", "Auth: request-otp p95"],
    ["auth_verify_otp_duration", "Auth: verify-otp p95"],
    ["conversation_list_duration", "Conversations: list p95"],
    ["message_send_duration", "Messages: send p95"],
    ["message_get_duration", "Messages: get p95"],
    ["http_req_duration", "Overall HTTP p95"],
  ];

  console.log("=== Paw HTTP API Load Test Summary ===");
  for (const [key, label] of metrics) {
    const val = data.metrics[key]
      ? `${data.metrics[key].values["p(95)"].toFixed(1)} ms`
      : "N/A";
    console.log(`  ${label}: ${val}`);
  }
  console.log("======================================");

  return {
    stdout: JSON.stringify(data, null, 2),
  };
}
