/**
 * Paw WebSocket Load Test — k6
 *
 * Scenario: 100 concurrent virtual users, each opens a WebSocket,
 * sends 10 messages, and measures server‐side latency.
 *
 * Run:
 *   k6 run k6/ws_load_test.js
 *   k6 run --env BASE_URL=ws://localhost:3000 k6/ws_load_test.js
 *
 * Prerequisites:
 *   - Paw server running with seeded test users
 *   - Pre‐generated JWT access tokens in k6/tokens.json (array of strings)
 *     OR set TOKEN env var for a single shared token
 */

import ws from "k6/ws";
import { check, sleep } from "k6";
import { Counter, Rate, Trend } from "k6/metrics";
import { uuidv4 } from "https://jslib.k6.io/k6-utils/1.4.0/index.js";

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

const BASE_URL = __ENV.BASE_URL || "ws://localhost:3000";
const TOKEN = __ENV.TOKEN || "REPLACE_WITH_JWT_ACCESS_TOKEN";
const CONVERSATION_ID =
  __ENV.CONVERSATION_ID || "00000000-0000-0000-0000-000000000001";
const MESSAGES_PER_VU = 10;

export const options = {
  scenarios: {
    websocket_load: {
      executor: "constant-vus",
      vus: 100,
      duration: "60s",
    },
  },
  thresholds: {
    ws_message_rtt: ["p(95)<200"], // p95 round‐trip < 200 ms
    ws_connect_duration: ["p(95)<500"], // p95 connect < 500 ms
    ws_message_success_rate: ["rate>0.99"], // 99%+ messages acknowledged
  },
};

// ---------------------------------------------------------------------------
// Custom metrics
// ---------------------------------------------------------------------------

const wsMessageRTT = new Trend("ws_message_rtt", true);
const wsConnectDuration = new Trend("ws_connect_duration", true);
const wsMessagesSent = new Counter("ws_messages_sent");
const wsMessagesReceived = new Counter("ws_messages_received");
const wsMessageSuccessRate = new Rate("ws_message_success_rate");
const wsConnectErrors = new Counter("ws_connect_errors");

// ---------------------------------------------------------------------------
// Main VU function
// ---------------------------------------------------------------------------

export default function () {
  const url = `${BASE_URL}/ws?token=${TOKEN}`;
  const connectStart = Date.now();

  const res = ws.connect(url, {}, function (socket) {
    const connectMs = Date.now() - connectStart;
    wsConnectDuration.add(connectMs);

    let helloReceived = false;
    let pendingMessages = new Map(); // idempotency_key → send timestamp
    let messagesSent = 0;

    socket.on("open", () => {
      // Server sends hello_ok on connect; no explicit connect frame needed
      // in current implementation (JWT validated on HTTP upgrade).
    });

    socket.on("message", (data) => {
      let frame;
      try {
        frame = JSON.parse(data);
      } catch (_) {
        return;
      }

      // Handle hello_ok — marks connection ready
      if (frame.type === "hello_ok") {
        helloReceived = true;
        sendNextBatch(socket, pendingMessages, messagesSent);
        return;
      }

      // Handle message_received — measure RTT if we sent it
      if (frame.type === "message_received") {
        wsMessagesReceived.add(1);
        // Use idempotency from our pending map (match by content prefix)
        for (const [key, sentAt] of pendingMessages) {
          if (frame.content && frame.content.startsWith(`k6-bench-${key.slice(0, 8)}`)) {
            wsMessageRTT.add(Date.now() - sentAt);
            wsMessageSuccessRate.add(1);
            pendingMessages.delete(key);
            break;
          }
        }
        return;
      }

      // Handle hello_error
      if (frame.type === "hello_error") {
        wsConnectErrors.add(1);
        socket.close();
      }
    });

    socket.on("error", () => {
      wsConnectErrors.add(1);
    });

    // Send messages once hello_ok received (or immediately for fallback)
    socket.setTimeout(() => {
      if (!helloReceived) {
        // Fallback: start sending even without explicit hello_ok
        sendNextBatch(socket, pendingMessages, messagesSent);
      }
    }, 2000);

    // Keep connection alive for the full scenario
    socket.setTimeout(() => {
      // Mark any unsent messages as failed
      for (const [key] of pendingMessages) {
        wsMessageSuccessRate.add(0);
      }
      socket.close();
    }, 55000); // Close before 60s scenario ends

    function sendNextBatch(sock, pending, startFrom) {
      for (let i = startFrom; i < MESSAGES_PER_VU; i++) {
        const idempotencyKey = uuidv4();
        const sendTs = Date.now();
        pending.set(idempotencyKey, sendTs);

        const frame = JSON.stringify({
          v: 1,
          type: "message_send",
          conversation_id: CONVERSATION_ID,
          content: `k6-bench-${idempotencyKey.slice(0, 8)}-vu${__VU}-iter${i}`,
          format: "plain",
          blocks: [],
          idempotency_key: idempotencyKey,
        });

        sock.send(frame);
        wsMessagesSent.add(1);
        messagesSent++;

        // Stagger messages: 500ms between each
        sleep(0.5);
      }
    }
  });

  check(res, {
    "WS status is 101": (r) => r && r.status === 101,
  });
}

// ---------------------------------------------------------------------------
// Lifecycle hooks
// ---------------------------------------------------------------------------

export function handleSummary(data) {
  const p95Rtt = data.metrics.ws_message_rtt
    ? data.metrics.ws_message_rtt.values["p(95)"]
    : "N/A";
  const p95Connect = data.metrics.ws_connect_duration
    ? data.metrics.ws_connect_duration.values["p(95)"]
    : "N/A";
  const sent = data.metrics.ws_messages_sent
    ? data.metrics.ws_messages_sent.values.count
    : 0;
  const received = data.metrics.ws_messages_received
    ? data.metrics.ws_messages_received.values.count
    : 0;

  console.log("=== Paw WebSocket Load Test Summary ===");
  console.log(`  VUs:             100`);
  console.log(`  Messages/VU:     ${MESSAGES_PER_VU}`);
  console.log(`  Total Sent:      ${sent}`);
  console.log(`  Total Received:  ${received}`);
  console.log(`  p95 RTT:         ${p95Rtt} ms`);
  console.log(`  p95 Connect:     ${p95Connect} ms`);
  console.log("=======================================");

  return {
    stdout: JSON.stringify(data, null, 2),
  };
}
