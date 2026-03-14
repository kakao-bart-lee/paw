import Foundation

struct PawConversationPreview: Identifiable, Equatable {
    let id: String
    var title: String
    var subtitle: String
    var unreadCount: Int
    var accent: String
}

struct PawMessagePreview: Identifiable, Equatable {
    enum Role: String {
        case me
        case peer
        case agent
    }

    let id: String
    let conversationID: String
    var author: String
    var body: String
    var role: Role
    var timestampLabel: String
}
