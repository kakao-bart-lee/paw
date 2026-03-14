import Foundation

enum PawApiError: Error, LocalizedError {
    case invalidURL
    case httpError(statusCode: Int, message: String)
    case decodingError(String)
    case networkError(String)

    var errorDescription: String? {
        switch self {
        case .invalidURL:
            return "Invalid URL"
        case .httpError(let statusCode, let message):
            return "Server error (\(statusCode)): \(message)"
        case .decodingError(let detail):
            return "Failed to parse response: \(detail)"
        case .networkError(let detail):
            return "Network error: \(detail)"
        }
    }
}

actor PawApiClient {
    let baseURL: String
    private var accessToken: String?

    init(baseURL: String = PawApiClient.defaultBaseURL) {
        self.baseURL = baseURL
    }

    static var defaultBaseURL: String {
        #if targetEnvironment(simulator)
        "http://localhost:38173"
        #else
        "http://localhost:38173"
        #endif
    }

    func setAccessToken(_ token: String?) {
        accessToken = token
    }

    // MARK: - Auth

    func requestOtp(phone: String) async throws -> [String: Any] {
        try await post(path: "/auth/request-otp", body: ["phone": phone])
    }

    func verifyOtp(phone: String, code: String) async throws -> [String: Any] {
        try await post(path: "/auth/verify-otp", body: ["phone": phone, "code": code])
    }

    func registerDevice(
        sessionToken: String,
        deviceName: String,
        ed25519PublicKey: String
    ) async throws -> [String: Any] {
        try await post(path: "/auth/register-device", body: [
            "session_token": sessionToken,
            "device_name": deviceName,
            "ed25519_public_key": ed25519PublicKey,
        ])
    }

    func getMe() async throws -> [String: Any] {
        try await get(path: "/users/me")
    }

    func updateMe(username: String, discoverableByPhone: Bool) async throws -> [String: Any] {
        try await request(method: "PATCH", path: "/users/me", body: [
            "username": username,
            "discoverable_by_phone": discoverableByPhone,
        ])
    }

    // MARK: - Conversations

    func getConversations() async throws -> [[String: Any]] {
        let result = try await get(path: "/conversations")
        guard let conversations = result["conversations"] as? [[String: Any]] else {
            return []
        }
        return conversations
    }

    func getMessages(conversationId: String) async throws -> [[String: Any]] {
        let result = try await get(path: "/conversations/\(conversationId)/messages")
        guard let messages = result["messages"] as? [[String: Any]] else {
            return []
        }
        return messages
    }

    func sendMessage(
        conversationId: String,
        content: String
    ) async throws -> [String: Any] {
        try await post(
            path: "/conversations/\(conversationId)/messages",
            body: [
                "content": content,
                "format": "plain",
                "idempotency_key": UUID().uuidString,
            ]
        )
    }

    // MARK: - HTTP helpers

    private func get(path: String) async throws -> [String: Any] {
        try await request(method: "GET", path: path, body: nil)
    }

    private func post(path: String, body: [String: Any]) async throws -> [String: Any] {
        try await request(method: "POST", path: path, body: body)
    }

    private func request(
        method: String,
        path: String,
        body: [String: Any]?
    ) async throws -> [String: Any] {
        guard let url = URL(string: baseURL.trimmingSuffix("/") + path) else {
            throw PawApiError.invalidURL
        }

        var urlRequest = URLRequest(url: url)
        urlRequest.httpMethod = method
        urlRequest.setValue("application/json", forHTTPHeaderField: "Content-Type")
        urlRequest.timeoutInterval = 15

        if let token = accessToken, !token.isEmpty {
            urlRequest.setValue("Bearer \(token)", forHTTPHeaderField: "Authorization")
        }

        if let body {
            urlRequest.httpBody = try JSONSerialization.data(withJSONObject: body)
        }

        let data: Data
        let response: URLResponse
        do {
            (data, response) = try await URLSession.shared.data(for: urlRequest)
        } catch {
            throw PawApiError.networkError(error.localizedDescription)
        }

        guard let httpResponse = response as? HTTPURLResponse else {
            throw PawApiError.networkError("Invalid response type")
        }

        let statusCode = httpResponse.statusCode

        if data.isEmpty && statusCode >= 200 && statusCode < 300 {
            return [:]
        }

        guard let json = try? JSONSerialization.jsonObject(with: data) as? [String: Any] else {
            if statusCode >= 200 && statusCode < 300 {
                return [:]
            }
            throw PawApiError.decodingError("Non-JSON response (HTTP \(statusCode))")
        }

        guard statusCode >= 200 && statusCode < 300 else {
            let message = (json["message"] as? String)
                ?? (json["error"] as? String)
                ?? "HTTP \(statusCode)"
            throw PawApiError.httpError(statusCode: statusCode, message: message)
        }

        return json
    }
}

private extension String {
    func trimmingSuffix(_ suffix: String) -> String {
        if hasSuffix(suffix) {
            return String(dropLast(suffix.count))
        }
        return self
    }
}
