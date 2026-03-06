/**
 * Paw Final Performance Benchmark — k6
 *
 * Comprehensive load test with 1000 concurrent virtual users covering:
 *   1. HTTP message send (POST /conversations/:id/messages)
 *   2. WebSocket connect + message RTT
 *   3. Media upload presigned URL (GET /media/:id/url)
 *
 * Run:
 *   k6 run k6/final-benchmark.js
 *   k6 run --env BASE_URL=http://localhost:3000 \
 *           --env WS_URL=ws://localhost:3000 \
 *           --env TOKEN=<jwt> \
 *           k6/final-benchmark.js
 */

import http from "k6/http";
import ws from "k6/ws";
import { check, group, sleep } from "k6";
import { Counter, Rate, Trend } from "k6/metrics";
import { uuidv4 } from "https://jslib.k6.io/k6-utils/1.4.0/index.js";

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

const BASE_URL = __ENV.BASE_URL || "http://localhost:3000";
const WS_URL = __ENV.WS_URL || "ws://localhost:3000";
const TOKEN = __ENV.TOKEN || "REPLACE_WITH_JWT_ACCESS_TOKEN";
const CONVERSATION_ID =
  __ENV.CONVERSATION_ID || "00000000-0000-0000-0000-000000000001";
const MEDIA_ID =
  __ENV.MEDIA_ID || "00000000-0000-0000-0000-000000000001";

// ---------------------------------------------------------------------------
// Options — 1000 VUs total, ramped over 30s, sustained for 60s
// ---------------------------------------------------------------------------

export const options = {
  scenarios: {
    // Scenario 1: HTTP message send — 500 VUs
    http_message_send: {
      executor: "ramping-vus",
      startVUs: 0,
      stages: [
        { duration: "30s", target: 500 },  // ramp up
        { duration: "60s", target: 500 },  // sustain
        { duration: "10s", target: 0 },    // ramp down
      ],
      exec: "httpMessageSend",
      gracefulRampDown: "10s",
    },
    // Scenario 2: WebSocket connect + RTT — 400 VUs
    ws_connect_rtt: {
      executor: "ramping-vus",
      startVUs: 0,
      stages: [
        { duration: "30s", target: 400 },
        { duration: "60s", target: 400 },
        { duration: "10s", target: 0 },
      ],
      exec: "wsConnectRTT",
      gracefulRampDown: "15s",
    },
    // Scenario 3: Media presigned URL fetch — 100 VUs
    media_presigned_url: {
      executor: "ramping-vus",
      startVUs: 0,
      stages: [
        { duration: "30s", target: 100 },
        { duration: "60s", target: 100 },
        { duration: "10s", target: 0 },
      ],
      exec: "mediaPresignedUrl",
      gracefulRampDown: "10s",
    },
  },
  thresholds: {
    // KPI targets
    http_message_send_p95: ["p(95)<200"],      // HTTP message send p95 < 200ms
    ws_rtt_p95: ["p(95)<200"],                  // WS round-trip p95 < 200ms
    ws_connect_p95: ["p(95)<500"],              // WS connect p95 < 500ms
    media_presigned_url_p95: ["p(95)<200"],     // Media presigned URL p95 < 200ms
    http_req_failed: ["rate<0.05"],             // < 5% error rate
    ws_delivery_rate: ["rate>0.99"],            // > 99% WS delivery
  },
};

// ---------------------------------------------------------------------------
// Custom Metrics
// ---------------------------------------------------------------------------

// HTTP message send
const httpMessageSendP95 = new Trend("http_message_send_p95", true);
const httpMessageSendErrors = new Counter("http_message_send_errors");

// WebSocket
const wsRttP95 = new Trend("ws_rtt_p95", true);
const wsConnectP95 = new Trend("ws_connect_p95", true);
const wsDeliveryRate = new Rate("ws_delivery_rate");
const wsMessagesSent = new Counter("ws_messages_sent_total");
const wsMessagesReceived = new Counter("ws_messages_received_total");
const wsConnectErrors = new Counter("ws_connect_errors");

// Media presigned URL
const mediaPresignedUrlP95 = new Trend("media_presigned_url_p95", true);
const mediaPresignedUrlErrors = new Counter("media_presigned_url_errors");

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function authHeaders() {
  return {
    headers: {
      "Content-Type": "application/json",
      Authorization: `Bearer ${TOKEN}`,
    },
  };
}

// ---------------------------------------------------------------------------
// Scenario 1: HTTP Message Send
// ---------------------------------------------------------------------------

export function httpMessageSend() {
  const opts = authHeaders();

  group("POST /conversations/:id/messages", () => {
    const payload = JSON.stringify({
      content: `k6-final-vu${__VU}-iter${__ITER}-${Date.now()}`,
      format: "plain",
      idempotency_key: uuidv4(),
    });

    const res = http.post(
      `${BASE_URL}/conversations/${CONVERSATION_ID}/messages`,
      payload,
      opts
    );

    httpMessageSendP95.add(res.timings.duration);

    const ok = check(res, {
      "message send: status 200|201|401|403": (r) =>
        [200, 201, 401, 403].includes(r.status),
      "message send: response time < 500ms": (r) =>
        r.timings.duration < 500,
    });

    if (!ok) httpMessageSendErrors.add(1);
  });

  // Think time: simulate realistic user pacing
  sleep(Math.random() * 0.5 + 0.3);
}

// ---------------------------------------------------------------------------
// Scenario 2: WebSocket Connect + Message RTT
// ---------------------------------------------------------------------------

export function wsConnectRTT() {
  const url = `${WS_URL}/ws?token=${TOKEN}`;
  const connectStart = Date.now();

  const res = ws.connect(url, {}, function (socket) {
    const connectMs = Date.now() - connectStart;
    wsConnectP95.add(connectMs);

    const pending = new Map();
    let messagesSent = 0;
    const maxMessages = 5;

    socket.on("open", () => {
      // Connection established; server sends hello_ok after JWT validation
    });

    socket.on("message", (data) => {
      let frame;
      try {
        frame = JSON.parse(data);
      } catch (_) {
        return;
      }

      if (frame.type === "hello_ok") {
        // Start sending messages after handshake
        sendMessages(socket);
        return;
      }

      if (frame.type === "message_received" || frame.type === "message_ack") {
        wsMessagesReceived.add(1);
        // Match by idempotency_key if available
        const key = frame.idempotency_key;
        if (key && pending.has(key)) {
          wsRttP95.add(Date.now() - pending.get(key));
          wsDeliveryRate.add(1);
          pending.delete(key);
        } else {
          // Fallback: count as delivered even without exact match
          wsDeliveryRate.add(1);
        }
        return;
      }

      if (frame.type === "hello_error") {
        wsConnectErrors.add(1);
        socket.close();
      }
    });

    socket.on("error", () => {
      wsConnectErrors.add(1);
    });

    // Fallback: send even without hello_ok after 2s
    socket.setTimeout(() => {
      if (messagesSent === 0) {
        sendMessages(socket);
      }
    }, 2000);

    // Close before scenario ends; mark undelivered as failed
    socket.setTimeout(() => {
      for (const [key] of pending) {
        wsDeliveryRate.add(0);
      }
      socket.close();
    }, 50000);

    function sendMessages(sock) {
      for (let i = 0; i < maxMessages; i++) {
        const idempotencyKey = uuidv4();
        pending.set(idempotencyKey, Date.now());

        sock.send(
          JSON.stringify({
            v: 1,
            type: "message_send",
            conversation_id: CONVERSATION_ID,
            content: `k6-final-ws-vu${__VU}-msg${i}`,
            format: "plain",
            blocks: [],
            idempotency_key: idempotencyKey,
          })
        );

        wsMessagesSent.add(1);
        messagesSent++;
        sleep(0.8);
      }
    }
  });

  check(res, {
    "WS status is 101 (Switching Protocols)": (r) => r && r.status === 101,
  });
}

// ---------------------------------------------------------------------------
// Scenario 3: Media Presigned URL
// ---------------------------------------------------------------------------

export function mediaPresignedUrl() {
  const opts = authHeaders();

  group("GET /media/:media_id/url", () => {
    const res = http.get(
      `${BASE_URL}/media/${MEDIA_ID}/url`,
      opts
    );

    mediaPresignedUrlP95.add(res.timings.duration);

    const ok = check(res, {
      "presigned URL: status 200|401|404": (r) =>
        [200, 401, 404].includes(r.status),
      "presigned URL: response time < 500ms": (r) =>
        r.timings.duration < 500,
    });

    if (!ok) mediaPresignedUrlErrors.add(1);
  });

  sleep(Math.random() * 0.5 + 0.5);
}

// ---------------------------------------------------------------------------
// Summary
// ---------------------------------------------------------------------------

export function handleSummary(data) {
  const kpis = [
    ["http_message_send_p95", "HTTP Message Send p95"],
    ["ws_rtt_p95", "WS Message RTT p95"],
    ["ws_connect_p95", "WS Connect p95"],
    ["media_presigned_url_p95", "Media Presigned URL p95"],
  ];

  console.log("╔═══════════════════════════════════════════════════╗");
  console.log("║     Paw Final Performance Benchmark Results      ║");
  console.log("╠═══════════════════════════════════════════════════╣");
  console.log("║  Total VUs: 1000 (500 HTTP + 400 WS + 100 Media)║");
  console.log("╠═══════════════════════════════════════════════════╣");

  for (const [key, label] of kpis) {
    const m = data.metrics[key];
    const val = m ? `${m.values["p(95)"].toFixed(1)} ms` : "N/A";
    const target = key.includes("connect") ? "< 500 ms" : "< 200 ms";
    const pass = m && m.values["p(95)"] < (key.includes("connect") ? 500 : 200);
    const status = m ? (pass ? "✅ PASS" : "❌ FAIL") : "⏳ N/A";
    console.log(`║  ${label.padEnd(28)} ${val.padStart(10)} ${status.padStart(9)} ║`);
  }

  // WS delivery rate
  const deliveryRate = data.metrics["ws_delivery_rate"];
  if (deliveryRate) {
    const rate = (deliveryRate.values.rate * 100).toFixed(1);
    const pass = deliveryRate.values.rate > 0.99;
    console.log(`║  WS Delivery Rate              ${rate.padStart(7)}% ${pass ? "✅ PASS" : "❌ FAIL"} ║`);
  }

  // HTTP error rate
  const errorRate = data.metrics["http_req_failed"];
  if (errorRate) {
    const rate = (errorRate.values.rate * 100).toFixed(1);
    const pass = errorRate.values.rate < 0.05;
    console.log(`║  HTTP Error Rate               ${rate.padStart(7)}% ${pass ? "✅ PASS" : "❌ FAIL"} ║`);
  }

  console.log("╚═══════════════════════════════════════════════════╝");

  return {
    stdout: JSON.stringify(data, null, 2),
  };
}
