import SwiftUI

struct ChatView: View {
    @EnvironmentObject private var coreManager: PawCoreManager

    @State private var activeLayer: LayerTab = .stream
    @State private var showingPresence = false
    @State private var activeConversationID: String?
    @State private var composerText = ""
    @State private var boundSignalIDs: Set<String> = ["oracle", "scribe"]

    @State private var searchQuery = ""
    @State private var showingSearchOverlay = false
    @State private var showingComposeOverlay = false
    @State private var composeMode: ComposeMode = .direct
    @State private var composeTitle = ""
    @State private var composeSubtitle = ""

    @State private var selectedSignal: SignalEntity?
    @State private var selectedSetting: SelfSetting?

    @State private var localConversations: [LocalConversation] = []
    @State private var localMessages: [String: [PawMessagePreview]] = [:]
    @State private var readReceiptsEnabled = false
    @State private var notificationsSelective = true
    @State private var appearanceVoid = true
    @State private var encryptionEnabled = true
    @State private var securityVerified = true

    var body: some View {
        ZStack {
            background

            if showingPresence, let conversation = selectedConversation {
                presenceLayer(conversation: conversation)
            } else {
                currentLayer
                    .transition(.opacity)
            }

            if showingSearchOverlay {
                searchOverlay
            }

            if showingComposeOverlay {
                composeOverlay
            }

            if let selectedSignal {
                signalOverlay(selectedSignal)
            }

            if let selectedSetting {
                selfDetailOverlay(selectedSetting)
            }
        }
        .accessibilityIdentifier(PawAccessibility.mainShell)
        .onAppear {
            ensureInitialSelection()
        }
    }

    private var background: some View {
        LinearGradient(
            colors: [PawTheme.background, PawTheme.backgroundGlow.opacity(0.18), PawTheme.background],
            startPoint: .bottom,
            endPoint: .top
        )
        .ignoresSafeArea()
    }

    @ViewBuilder
    private var currentLayer: some View {
        switch activeLayer {
        case .stream:
            streamLayer
        case .signals:
            signalsLayer
        case .selfLayer:
            selfLayer
        }
    }

    private var streamLayer: some View {
        VStack(spacing: 0) {
            header(
                title: "STREAM",
                subtitle: nil,
                trailing: AnyView(
                    HStack(spacing: 18) {
                        Button {
                            showingSearchOverlay = true
                        } label: {
                            Image(systemName: "magnifyingglass")
                                .font(.system(size: 16, weight: .light, design: .monospaced))
                                .foregroundStyle(PawTheme.subtleText)
                        }
                        .buttonStyle(.plain)

                        Button {
                            showingComposeOverlay = true
                        } label: {
                            Image(systemName: "plus")
                                .font(.system(size: 18, weight: .light, design: .monospaced))
                                .foregroundStyle(PawTheme.subtleText)
                        }
                        .buttonStyle(.plain)
                    }
                )
            )

            ScrollView(showsIndicators: false) {
                VStack(spacing: 0) {
                    ForEach(allConversations) { entity in
                        Button {
                            openConversation(entity)
                        } label: {
                            streamRow(entity)
                        }
                        .buttonStyle(.plain)
                    }
                }
                .padding(.top, 22)
                .padding(.bottom, 110)
            }

            bottomNavigation
        }
    }

    private func streamRow(_ entity: ConversationDisplay) -> some View {
        HStack(alignment: .top, spacing: 14) {
            Rectangle()
                .fill(entity.color)
                .frame(width: 3)
                .opacity(0.9)

            SignatureView(kind: entity.signature, color: entity.color)
                .frame(width: 32, height: 32)
                .padding(.top, 2)

            VStack(alignment: .leading, spacing: 4) {
                HStack(alignment: .firstTextBaseline, spacing: 8) {
                    Text(entity.title)
                        .font(PawTypography.titleMedium)
                        .foregroundStyle(PawTheme.strongText)
                    Text(entity.time)
                        .font(PawTypography.labelSmall)
                        .foregroundStyle(PawTheme.mutedText)
                    if entity.isSignal {
                        Text("SIGNAL")
                            .font(PawTypography.labelSmall)
                            .tracking(2)
                            .foregroundStyle(PawTheme.teal.opacity(0.8))
                    }
                    if entity.isCollective {
                        Text("COLLECTIVE")
                            .font(PawTypography.labelSmall)
                            .tracking(2)
                            .foregroundStyle(PawTheme.lavender.opacity(0.7))
                    }
                }
                Text(entity.preview)
                    .font(PawTypography.bodySmall)
                    .foregroundStyle(PawTheme.mutedText)
                    .lineLimit(1)
            }

            Spacer()

            Circle()
                .fill(entity.color)
                .frame(width: 7, height: 7)
                .padding(.top, 8)
                .opacity(entity.highlighted ? 1 : 0.7)
        }
        .padding(.horizontal, 22)
        .padding(.vertical, 18)
        .background(Color.clear)
        .overlay(alignment: .bottom) {
            Rectangle()
                .fill(PawTheme.outline)
                .frame(height: 1)
                .padding(.leading, 54)
        }
    }

    private var signalsLayer: some View {
        VStack(spacing: 0) {
            header(title: "SIGNALS", subtitle: "AI entities that can be bound to your presence")

            ScrollView(showsIndicators: false) {
                VStack(spacing: 0) {
                    ForEach(signalEntities) { signal in
                        Button {
                            selectedSignal = signal
                        } label: {
                            signalRow(signal)
                        }
                        .buttonStyle(.plain)
                    }
                }
                .padding(.top, 26)
                .padding(.bottom, 110)
            }

            bottomNavigation
        }
    }

    private func signalRow(_ signal: SignalEntity) -> some View {
        HStack(alignment: .top, spacing: 14) {
            Rectangle()
                .fill(PawTheme.teal)
                .frame(width: 3)
                .opacity(0.7)

            VStack(alignment: .leading, spacing: 8) {
                HStack(spacing: 8) {
                    Text(signal.name)
                        .font(PawTypography.titleMedium)
                        .foregroundStyle(PawTheme.strongText)
                    Text(signal.domain.uppercased())
                        .font(PawTypography.labelSmall)
                        .tracking(2)
                        .foregroundStyle(PawTheme.mutedText)
                }
                Text(signal.essence)
                    .font(PawTypography.bodySmall)
                    .italic()
                    .foregroundStyle(PawTheme.subtleText)
                FlowLayout(spacing: 8) {
                    ForEach(signal.abilities, id: \.self) { ability in
                        Text(ability.uppercased())
                            .font(PawTypography.labelSmall)
                            .tracking(1.5)
                            .foregroundStyle(PawTheme.teal.opacity(0.55))
                            .padding(.horizontal, 8)
                            .padding(.vertical, 6)
                            .overlay(
                                Rectangle().stroke(PawTheme.teal.opacity(0.12), lineWidth: 1)
                            )
                    }
                }
            }

            Spacer()

            Group {
                if boundSignalIDs.contains(signal.id) {
                    Circle()
                        .stroke(PawTheme.amber.opacity(0.55), lineWidth: 1)
                        .frame(width: 24, height: 24)
                        .overlay(
                            Image(systemName: "checkmark")
                                .font(.system(size: 11, weight: .regular))
                                .foregroundStyle(PawTheme.amber)
                        )
                } else {
                    Circle()
                        .stroke(PawTheme.outline, lineWidth: 1)
                        .frame(width: 24, height: 24)
                }
            }
            .padding(.top, 2)
        }
        .padding(.horizontal, 22)
        .padding(.vertical, 18)
        .overlay(alignment: .bottom) {
            Rectangle()
                .fill(PawTheme.outline)
                .frame(height: 1)
                .padding(.leading, 38)
        }
    }

    private var selfLayer: some View {
        VStack(spacing: 0) {
            header(title: "SELF")

            ScrollView(showsIndicators: false) {
                VStack(spacing: 0) {
                    profileOrb
                        .padding(.top, 42)

                    Text(coreManager.preview.auth.username.ifEmpty("User"))
                        .font(PawTypography.headlineMedium)
                        .foregroundStyle(PawTheme.strongText)
                        .padding(.top, 18)
                    Text(coreManager.preview.auth.phone.ifEmpty("+82 10-****-5678"))
                        .font(PawTypography.bodySmall)
                        .foregroundStyle(PawTheme.mutedText)
                        .padding(.top, 6)

                    HStack(spacing: 30) {
                        profileStat(number: boundSignalIDs.count, title: "signals")
                        Rectangle().fill(PawTheme.outline).frame(width: 1, height: 54)
                        profileStat(number: allConversations.count + 3, title: "connections")
                    }
                    .padding(.top, 42)

                    VStack(alignment: .leading, spacing: 0) {
                        Text("CONFIGURATION")
                            .font(PawTypography.labelSmall)
                            .tracking(2)
                            .foregroundStyle(PawTheme.mutedText)
                            .padding(.bottom, 14)

                        ForEach(selfSettings) { item in
                            Button {
                                selectedSetting = item
                            } label: {
                                selfSettingRow(item)
                            }
                            .buttonStyle(.plain)
                        }
                    }
                    .padding(.horizontal, 22)
                    .padding(.top, 34)

                    Button {
                        coreManager.logout()
                        activeConversationID = nil
                        showingPresence = false
                    } label: {
                        Text("DISCONNECT")
                            .font(PawTypography.labelMedium)
                            .tracking(3)
                            .foregroundStyle(PawTheme.danger.opacity(0.8))
                            .frame(maxWidth: .infinity)
                            .padding(.vertical, 14)
                            .overlay(Rectangle().stroke(PawTheme.danger.opacity(0.22), lineWidth: 1))
                    }
                    .buttonStyle(.plain)
                    .padding(.horizontal, 22)
                    .padding(.top, 28)
                    .padding(.bottom, 110)
                }
            }

            bottomNavigation
        }
    }

    private var profileOrb: some View {
        ZStack {
            Circle().stroke(PawTheme.amber.opacity(0.16), lineWidth: 1).frame(width: 84, height: 84)
            Circle().stroke(PawTheme.amber.opacity(0.22), lineWidth: 1).frame(width: 68, height: 68)
            Circle().fill(PawTheme.amberSoft).frame(width: 52, height: 52)
            Text(coreManager.preview.auth.username.ifEmpty("U").prefix(1).uppercased())
                .font(PawTypography.headlineMedium)
                .foregroundStyle(PawTheme.amber)
        }
    }

    private func profileStat(number: Int, title: String) -> some View {
        VStack(spacing: 6) {
            Text("\(number)")
                .font(PawTypography.headlineLarge)
                .foregroundStyle(PawTheme.strongText)
            Text(title.uppercased())
                .font(PawTypography.labelSmall)
                .tracking(2)
                .foregroundStyle(PawTheme.mutedText)
        }
    }

    private func selfSettingRow(_ item: SelfSetting) -> some View {
        HStack(spacing: 12) {
            Image(systemName: item.icon)
                .font(.system(size: 14, weight: .light))
                .foregroundStyle(PawTheme.mutedText)
                .frame(width: 18)
            Text(item.title)
                .font(PawTypography.titleMedium)
                .foregroundStyle(PawTheme.strongText)
            Spacer()
            Text(item.value(in: selfContext))
                .font(PawTypography.bodySmall)
                .foregroundStyle(PawTheme.mutedText)
            Circle()
                .fill(item.active(in: selfContext) ? PawTheme.amber : PawTheme.mutedText.opacity(0.35))
                .frame(width: 7, height: 7)
        }
        .padding(.vertical, 16)
        .overlay(alignment: .bottom) {
            Rectangle()
                .fill(PawTheme.outline)
                .frame(height: 1)
        }
    }

    private func presenceLayer(conversation: ConversationDisplay) -> some View {
        VStack(spacing: 0) {
            HStack(spacing: 12) {
                Button {
                    showingPresence = false
                } label: {
                    Image(systemName: "arrow.left")
                        .font(.system(size: 16, weight: .regular, design: .monospaced))
                        .foregroundStyle(PawTheme.subtleText)
                }
                .buttonStyle(.plain)

                VStack(alignment: .leading, spacing: 4) {
                    Text(conversation.title)
                        .font(PawTypography.titleMedium)
                        .foregroundStyle(PawTheme.strongText)
                    HStack(spacing: 6) {
                        Image(systemName: "lock.fill")
                            .font(.system(size: 10))
                            .foregroundStyle(PawTheme.amber.opacity(0.7))
                        Text("ENCRYPTED")
                            .font(PawTypography.labelSmall)
                            .tracking(2)
                            .foregroundStyle(PawTheme.mutedText)
                    }
                }

                Spacer()

                Circle()
                    .fill(conversation.color)
                    .frame(width: 8, height: 8)
            }
            .padding(.horizontal, 20)
            .padding(.top, 14)
            .padding(.bottom, 18)
            .overlay(alignment: .bottom) { Rectangle().fill(PawTheme.outline).frame(height: 1) }

            ScrollView(showsIndicators: false) {
                VStack(spacing: 22) {
                    ForEach(messagesForActiveConversation) { message in
                        presenceMessage(message)
                    }
                }
                .padding(.horizontal, 20)
                .padding(.top, 26)
                .padding(.bottom, 110)
                .accessibilityIdentifier(PawAccessibility.messageList)
            }

            HStack(spacing: 12) {
                TextField("speak...", text: $composerText)
                    .font(PawTypography.bodyMedium)
                    .foregroundStyle(PawTheme.strongText)
                    .accessibilityIdentifier(PawAccessibility.composer)
                Button {
                    sendMessage()
                } label: {
                    Text("SEND")
                        .font(PawTypography.labelMedium)
                        .tracking(2)
                        .foregroundStyle(PawTheme.mutedText)
                }
                .buttonStyle(.plain)
                .accessibilityIdentifier(PawAccessibility.sendMessageButton)
            }
            .padding(.horizontal, 20)
            .padding(.vertical, 16)
            .overlay(alignment: .top) { Rectangle().fill(PawTheme.outline).frame(height: 1) }
        }
    }

    private func presenceMessage(_ message: PawMessagePreview) -> some View {
        VStack(alignment: alignment(for: message.role), spacing: 8) {
            if message.role == .agent {
                Text("SIGNAL")
                    .font(PawTypography.labelSmall)
                    .tracking(2)
                    .foregroundStyle(PawTheme.teal.opacity(0.55))
            }
            Text(message.body)
                .font(PawTypography.bodyLarge)
                .foregroundStyle(textColor(for: message.role))
                .italic(message.role == .agent)
                .multilineTextAlignment(textAlignment(for: message.role))
                .frame(maxWidth: .infinity, alignment: frameAlignment(for: message.role))
            Rectangle()
                .fill(lineColor(for: message.role))
                .frame(width: lineWidth(for: message.role), height: 1)
                .frame(maxWidth: .infinity, alignment: frameAlignment(for: message.role))
        }
    }

    private var bottomNavigation: some View {
        HStack {
            ForEach(LayerTab.allCases, id: \.self) { tab in
                Button {
                    activeLayer = tab
                    showingPresence = false
                } label: {
                    Text(tab.title)
                        .font(PawTypography.labelMedium)
                        .tracking(3)
                        .foregroundStyle(activeLayer == tab ? PawTheme.strongText : PawTheme.mutedText)
                        .frame(maxWidth: .infinity)
                        .padding(.vertical, 14)
                }
                .buttonStyle(.plain)
                .accessibilityIdentifier(tab.accessibilityIdentifier)
            }
        }
        .padding(.horizontal, 22)
        .padding(.top, 12)
        .padding(.bottom, 18)
        .background(PawTheme.background)
        .overlay(alignment: .top) {
            Rectangle().fill(PawTheme.outline).frame(height: 1)
        }
    }

    private func header(title: String, subtitle: String? = nil, trailing: AnyView? = nil) -> some View {
        VStack(alignment: .leading, spacing: 10) {
            HStack {
                Text(title)
                    .font(PawTypography.labelMedium)
                    .tracking(4)
                    .foregroundStyle(title == "STREAM" ? PawTheme.strongText : PawTheme.mutedText)
                    .accessibilityIdentifier(title == "STREAM" ? PawAccessibility.chatListTitle : "paw.layer.\(title.lowercased())")
                Spacer()
                trailing
            }
            if let subtitle {
                Text(subtitle)
                    .font(PawTypography.bodySmall)
                    .foregroundStyle(PawTheme.mutedText)
            }
        }
        .padding(.horizontal, 22)
        .padding(.top, 20)
        .padding(.bottom, 4)
    }

    private var searchOverlay: some View {
        overlayShell(title: "SEARCH THE STREAM") {
            VStack(alignment: .leading, spacing: 18) {
                pawOverlayField(title: "query", text: $searchQuery, placeholder: "name / fragment")

                if filteredConversations.isEmpty {
                    emptyState(title: "No matching presence", detail: "Try a shorter query or clear the field.", identifier: "paw.stream.search.empty")
                } else {
                    VStack(spacing: 0) {
                        ForEach(filteredConversations) { conversation in
                            Button {
                                showingSearchOverlay = false
                                openConversation(conversation)
                            } label: {
                                streamRow(conversation)
                            }
                            .buttonStyle(.plain)
                        }
                    }
                }
            }
        }
    }

    private var composeOverlay: some View {
        overlayShell(title: composeMode == .direct ? "WEAVE A DIRECT THREAD" : "WEAVE A COLLECTIVE") {
            VStack(alignment: .leading, spacing: 18) {
                HStack(spacing: 10) {
                    overlayChip("direct", selected: composeMode == .direct) { composeMode = .direct }
                    overlayChip("collective", selected: composeMode == .collective) { composeMode = .collective }
                }

                pawOverlayField(title: composeMode == .direct ? "target" : "collective", text: $composeTitle, placeholder: composeMode == .direct ? "who are you calling in?" : "group or ritual name")
                pawOverlayField(title: "opening fragment", text: $composeSubtitle, placeholder: "set the tone of the first echo")

                if !allConversations.isEmpty {
                    VStack(alignment: .leading, spacing: 10) {
                        Text("KNOWN PRESENCES")
                            .font(PawTypography.labelSmall)
                            .tracking(2)
                            .foregroundStyle(PawTheme.mutedText)
                        ScrollView(.horizontal, showsIndicators: false) {
                            HStack(spacing: 10) {
                                ForEach(allConversations.prefix(4)) { conversation in
                                    Button {
                                        composeTitle = conversation.title
                                        composeSubtitle = conversation.preview
                                    } label: {
                                        Text(conversation.title)
                                            .font(PawTypography.bodySmall)
                                            .foregroundStyle(PawTheme.strongText)
                                            .padding(.horizontal, 10)
                                            .padding(.vertical, 8)
                                            .overlay(Rectangle().stroke(PawTheme.outline, lineWidth: 1))
                                    }
                                    .buttonStyle(.plain)
                                }
                            }
                        }
                    }
                }

                Button {
                    createConversationFromComposer()
                } label: {
                    Text("WEAVE")
                }
                .buttonStyle(PawPrimaryButtonStyle(enabled: !composeTitle.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty))
                .disabled(composeTitle.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty)
            }
        }
    }

    private func signalOverlay(_ signal: SignalEntity) -> some View {
        overlayShell(title: signal.name.uppercased()) {
            VStack(alignment: .leading, spacing: 18) {
                metadataLine("domain", signal.domain.uppercased())
                Text(signal.essence)
                    .font(PawTypography.bodyMedium)
                    .italic()
                    .foregroundStyle(PawTheme.subtleText)

                VStack(alignment: .leading, spacing: 8) {
                    Text("BINDING PERMISSIONS")
                        .font(PawTypography.labelSmall)
                        .tracking(2)
                        .foregroundStyle(PawTheme.mutedText)
                    permissionLine("- access to conversation context")
                    permissionLine("- ability to process and respond")
                    permissionLine("- memory of interactions")
                }

                FlowLayout(spacing: 8) {
                    ForEach(signal.abilities, id: \.self) { ability in
                        Text(ability.uppercased())
                            .font(PawTypography.labelSmall)
                            .tracking(1.5)
                            .foregroundStyle(PawTheme.teal.opacity(0.55))
                            .padding(.horizontal, 8)
                            .padding(.vertical, 6)
                            .overlay(Rectangle().stroke(PawTheme.teal.opacity(0.12), lineWidth: 1))
                    }
                }

                Button {
                    toggleSignal(signal)
                    selectedSignal = nil
                } label: {
                    Text(boundSignalIDs.contains(signal.id) ? "UNBIND SIGNAL" : "BIND SIGNAL")
                }
                .buttonStyle(PawPrimaryButtonStyle(enabled: true))
            }
        }
    }

    private func selfDetailOverlay(_ item: SelfSetting) -> some View {
        overlayShell(title: item.title.uppercased()) {
            VStack(alignment: .leading, spacing: 18) {
                Text(item.detail)
                    .font(PawTypography.bodyMedium)
                    .foregroundStyle(PawTheme.subtleText)

                switch item.kind {
                case .encryption:
                    Toggle(isOn: $encryptionEnabled) {
                        Text("end-to-end encryption")
                            .font(PawTypography.bodySmall)
                            .foregroundStyle(PawTheme.strongText)
                    }
                    .tint(PawTheme.amber)
                case .securityKey:
                    Toggle(isOn: $securityVerified) {
                        Text("device key verified")
                            .font(PawTypography.bodySmall)
                            .foregroundStyle(PawTheme.strongText)
                    }
                    .tint(PawTheme.amber)
                case .readReceipts:
                    Toggle(isOn: $readReceiptsEnabled) {
                        Text("share read state")
                            .font(PawTypography.bodySmall)
                            .foregroundStyle(PawTheme.strongText)
                    }
                    .tint(PawTheme.amber)
                case .notifications:
                    Toggle(isOn: $notificationsSelective) {
                        Text("selective notifications")
                            .font(PawTypography.bodySmall)
                            .foregroundStyle(PawTheme.strongText)
                    }
                    .tint(PawTheme.amber)
                case .appearance:
                    Toggle(isOn: $appearanceVoid) {
                        Text("void appearance")
                            .font(PawTypography.bodySmall)
                            .foregroundStyle(PawTheme.strongText)
                    }
                    .tint(PawTheme.amber)
                }

                NoticeCard(title: "state", detail: item.value(in: selfContext), tone: .info)
            }
        }
    }

    private func overlayShell<Content: View>(title: String, @ViewBuilder content: () -> Content) -> some View {
        ZStack {
            Color.black.opacity(0.72)
                .ignoresSafeArea()
                .onTapGesture {
                    closeOverlays()
                }

            VStack(alignment: .leading, spacing: 22) {
                HStack {
                    Text(title)
                        .font(PawTypography.labelMedium)
                        .tracking(4)
                        .foregroundStyle(PawTheme.strongText)
                    Spacer()
                    Button {
                        closeOverlays()
                    } label: {
                        Image(systemName: "xmark")
                            .font(.system(size: 14, weight: .light, design: .monospaced))
                            .foregroundStyle(PawTheme.subtleText)
                    }
                    .buttonStyle(.plain)
                }
                content()
            }
            .padding(22)
            .frame(maxWidth: 320)
            .background(PawTheme.surface1)
            .overlay(Rectangle().stroke(PawTheme.outline, lineWidth: 1))
        }
    }

    private func pawOverlayField(title: String, text: Binding<String>, placeholder: String) -> some View {
        VStack(alignment: .leading, spacing: 8) {
            Text(title.uppercased())
                .font(PawTypography.labelSmall)
                .tracking(2)
                .foregroundStyle(PawTheme.mutedText)
            TextField(placeholder, text: text)
                .font(PawTypography.bodyMedium)
                .foregroundStyle(PawTheme.strongText)
                .padding(.vertical, 10)
                .overlay(alignment: .bottom) { Rectangle().fill(PawTheme.outline).frame(height: 1) }
        }
    }

    private func overlayChip(_ title: String, selected: Bool, action: @escaping () -> Void) -> some View {
        Button(action: action) {
            Text(title.uppercased())
                .font(PawTypography.labelSmall)
                .tracking(2)
                .foregroundStyle(selected ? PawTheme.strongText : PawTheme.mutedText)
                .padding(.horizontal, 10)
                .padding(.vertical, 8)
                .overlay(Rectangle().stroke(selected ? PawTheme.teal.opacity(0.24) : PawTheme.outline, lineWidth: 1))
        }
        .buttonStyle(.plain)
    }

    private func permissionLine(_ text: String) -> some View {
        Text(text)
            .font(PawTypography.bodySmall)
            .foregroundStyle(PawTheme.mutedText)
    }

    private func closeOverlays() {
        showingSearchOverlay = false
        showingComposeOverlay = false
        selectedSignal = nil
        selectedSetting = nil
    }

    private func ensureInitialSelection() {
        guard activeConversationID == nil else { return }
        if let first = allConversations.first {
            activeConversationID = first.id
            if first.source == .core {
                coreManager.selectConversation(first.id)
            }
        }
    }

    private func openConversation(_ conversation: ConversationDisplay) {
        activeConversationID = conversation.id
        if conversation.source == .core {
            coreManager.selectConversation(conversation.id)
        }
        showingPresence = true
        closeOverlays()
    }

    private func createConversationFromComposer() {
        let trimmedTitle = composeTitle.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !trimmedTitle.isEmpty else { return }
        let preview = composeSubtitle.trimmingCharacters(in: .whitespacesAndNewlines).ifEmpty(composeMode == .direct ? "new direct thread" : "new collective assembled")
        let id = "local-\(UUID().uuidString)"
        let signature: SignatureKind = composeMode == .direct ? .sine : .wave
        let color: Color = composeMode == .direct ? PawTheme.coral : PawTheme.lavender
        let local = LocalConversation(
            id: id,
            title: trimmedTitle,
            preview: preview,
            time: "now",
            color: color,
            signature: signature,
            isSignal: false,
            isCollective: composeMode == .collective,
            highlighted: true
        )
        localConversations.insert(local, at: 0)
        localMessages[id] = [
            PawMessagePreview(
                id: "\(id)-seed",
                conversationID: id,
                author: composeMode == .collective ? "Collective" : trimmedTitle,
                body: preview,
                role: .peer,
                timestampLabel: "now"
            )
        ]
        composeTitle = ""
        composeSubtitle = ""
        showingComposeOverlay = false
        openConversation(local.display)
    }

    private func sendMessage() {
        let text = composerText.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !text.isEmpty, let activeConversationID else { return }
        composerText = ""

        if let index = localConversations.firstIndex(where: { $0.id == activeConversationID }) {
            var messages = localMessages[activeConversationID] ?? []
            messages.append(
                PawMessagePreview(
                    id: "\(activeConversationID)-\(messages.count)-me",
                    conversationID: activeConversationID,
                    author: coreManager.preview.auth.username.ifEmpty("User"),
                    body: text,
                    role: .me,
                    timestampLabel: "now"
                )
            )
            messages.append(
                PawMessagePreview(
                    id: "\(activeConversationID)-\(messages.count)-signal",
                    conversationID: activeConversationID,
                    author: "Signal",
                    body: "based on your fragment, the stream now holds a new thread.",
                    role: .agent,
                    timestampLabel: "now"
                )
            )
            localMessages[activeConversationID] = messages
            localConversations[index].preview = messages.last?.body ?? text
            localConversations[index].time = "now"
        } else {
            coreManager.sendChatMessage(text)
        }
    }

    private var allConversations: [ConversationDisplay] {
        var result = coreConversations
        result.insert(contentsOf: localConversations.map(\.display), at: 0)
        return result
    }

    private var filteredConversations: [ConversationDisplay] {
        let query = searchQuery.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !query.isEmpty else { return allConversations }
        return allConversations.filter {
            $0.title.localizedCaseInsensitiveContains(query) ||
            $0.preview.localizedCaseInsensitiveContains(query)
        }
    }

    private var coreConversations: [ConversationDisplay] {
        let palette: [Color] = [PawTheme.teal, PawTheme.coral, PawTheme.lavender, PawTheme.teal, PawTheme.coral]
        let signatures: [SignatureKind] = [.pulse, .sine, .wave, .fractal, .sine]
        return coreManager.preview.conversations.enumerated().map { index, conversation in
            ConversationDisplay(
                id: conversation.id,
                title: conversation.title,
                preview: conversation.subtitle.ifEmpty(index == 0 ? "ready to assist" : "the meeting went better than expected"),
                time: index == 0 ? "active" : index == 1 ? "now" : "30m",
                color: palette[index % palette.count],
                signature: signatures[index % signatures.count],
                isSignal: conversation.id.contains("agent") || conversation.title.localizedCaseInsensitiveContains("agent"),
                isCollective: conversation.title.localizedCaseInsensitiveContains("void"),
                highlighted: index == 0,
                source: .core
            )
        }
    }

    private var selectedConversation: ConversationDisplay? {
        guard let activeConversationID else { return nil }
        return allConversations.first(where: { $0.id == activeConversationID })
    }

    private var messagesForActiveConversation: [PawMessagePreview] {
        guard let activeConversationID else { return [] }
        if let local = localMessages[activeConversationID] {
            return local
        }
        return coreManager.preview.messages
    }

    private var signalEntities: [SignalEntity] {
        [
            SignalEntity(id: "oracle", name: "Oracle", domain: "analysis", essence: "Sees patterns in chaos", abilities: ["pattern recognition", "prediction", "synthesis"]),
            SignalEntity(id: "scribe", name: "Scribe", domain: "creation", essence: "Transforms thought to word", abilities: ["writing", "translation", "summarization"]),
            SignalEntity(id: "sentinel", name: "Sentinel", domain: "security", essence: "Guards the threshold", abilities: ["encryption", "verification", "protection"]),
            SignalEntity(id: "weaver", name: "Weaver", domain: "integration", essence: "Connects disparate threads", abilities: ["automation", "linking"])
        ]
    }

    private var selfSettings: [SelfSetting] {
        [
            SelfSetting(kind: .encryption, icon: "lock", title: "Encryption", detail: "Control whether every active thread is treated as end-to-end sealed by default."),
            SelfSetting(kind: .securityKey, icon: "shield", title: "Security Key", detail: "Inspect the trust state of your device signature and recovery path."),
            SelfSetting(kind: .readReceipts, icon: "eye", title: "Read Receipts", detail: "Choose whether the stream reveals that you have seen a message."),
            SelfSetting(kind: .notifications, icon: "bell", title: "Notifications", detail: "Keep alerts selective so the quiet tone of the app remains intact."),
            SelfSetting(kind: .appearance, icon: "moon", title: "Appearance", detail: "Preserve the void-mode look when new surfaces are introduced.")
        ]
    }

    private var selfContext: SelfContext {
        SelfContext(
            encryptionEnabled: encryptionEnabled,
            securityVerified: securityVerified,
            readReceiptsEnabled: readReceiptsEnabled,
            notificationsSelective: notificationsSelective,
            appearanceVoid: appearanceVoid
        )
    }

    private func toggleSignal(_ signal: SignalEntity) {
        if boundSignalIDs.contains(signal.id) {
            boundSignalIDs.remove(signal.id)
        } else {
            boundSignalIDs.insert(signal.id)
        }
    }

    private func alignment(for role: PawMessagePreview.Role) -> HorizontalAlignment {
        switch role {
        case .me: .trailing
        case .peer: .leading
        case .agent: .center
        }
    }

    private func frameAlignment(for role: PawMessagePreview.Role) -> Alignment {
        switch role {
        case .me: .trailing
        case .peer: .leading
        case .agent: .center
        }
    }

    private func textAlignment(for role: PawMessagePreview.Role) -> TextAlignment {
        switch role {
        case .me: .trailing
        case .peer: .leading
        case .agent: .center
        }
    }

    private func textColor(for role: PawMessagePreview.Role) -> Color {
        switch role {
        case .me: PawTheme.strongText
        case .peer: PawTheme.strongText.opacity(0.9)
        case .agent: PawTheme.teal.opacity(0.66)
        }
    }

    private func lineColor(for role: PawMessagePreview.Role) -> Color {
        switch role {
        case .me: PawTheme.outline
        case .peer: PawTheme.outline
        case .agent: PawTheme.teal.opacity(0.22)
        }
    }

    private func lineWidth(for role: PawMessagePreview.Role) -> CGFloat {
        switch role {
        case .me: 180
        case .peer: 180
        case .agent: 140
        }
    }
}

private enum LayerTab: CaseIterable {
    case stream
    case signals
    case selfLayer

    var title: String {
        switch self {
        case .stream: "STREAM"
        case .signals: "SIGNALS"
        case .selfLayer: "SELF"
        }
    }

    var accessibilityIdentifier: String {
        switch self {
        case .stream: PawAccessibility.mainTabChat
        case .signals: PawAccessibility.mainTabAgent
        case .selfLayer: PawAccessibility.mainTabSettings
        }
    }
}

private enum ComposeMode {
    case direct
    case collective
}

private enum ConversationSource {
    case core
    case local
}

private struct ConversationDisplay: Identifiable {
    let id: String
    let title: String
    let preview: String
    let time: String
    let color: Color
    let signature: SignatureKind
    let isSignal: Bool
    let isCollective: Bool
    let highlighted: Bool
    let source: ConversationSource
}

private struct LocalConversation: Identifiable {
    let id: String
    var title: String
    var preview: String
    var time: String
    var color: Color
    var signature: SignatureKind
    var isSignal: Bool
    var isCollective: Bool
    var highlighted: Bool

    var display: ConversationDisplay {
        ConversationDisplay(
            id: id,
            title: title,
            preview: preview,
            time: time,
            color: color,
            signature: signature,
            isSignal: isSignal,
            isCollective: isCollective,
            highlighted: highlighted,
            source: .local
        )
    }
}

private struct SignalEntity: Identifiable {
    let id: String
    let name: String
    let domain: String
    let essence: String
    let abilities: [String]
}

private enum SelfSettingKind {
    case encryption
    case securityKey
    case readReceipts
    case notifications
    case appearance
}

private struct SelfSetting: Identifiable {
    let id = UUID()
    let kind: SelfSettingKind
    let icon: String
    let title: String
    let detail: String

    func value(in context: SelfContext) -> String {
        switch kind {
        case .encryption:
            context.encryptionEnabled ? "End-to-end" : "Ambient"
        case .securityKey:
            context.securityVerified ? "Verified" : "Needs attention"
        case .readReceipts:
            context.readReceiptsEnabled ? "Visible" : "Hidden"
        case .notifications:
            context.notificationsSelective ? "Selective" : "Full"
        case .appearance:
            context.appearanceVoid ? "Void" : "Standard"
        }
    }

    func active(in context: SelfContext) -> Bool {
        switch kind {
        case .encryption: context.encryptionEnabled
        case .securityKey: context.securityVerified
        case .readReceipts: context.readReceiptsEnabled
        case .notifications: context.notificationsSelective
        case .appearance: context.appearanceVoid
        }
    }
}

private struct SelfContext {
    let encryptionEnabled: Bool
    let securityVerified: Bool
    let readReceiptsEnabled: Bool
    let notificationsSelective: Bool
    let appearanceVoid: Bool
}

private enum SignatureKind {
    case sine
    case pulse
    case wave
    case fractal
}

private struct SignatureView: View {
    let kind: SignatureKind
    let color: Color

    var body: some View {
        GeometryReader { proxy in
            Path { path in
                let w = proxy.size.width
                let h = proxy.size.height
                switch kind {
                case .sine:
                    path.move(to: CGPoint(x: 0, y: h * 0.55))
                    path.addCurve(to: CGPoint(x: w * 0.5, y: h * 0.55), control1: CGPoint(x: w * 0.15, y: h * 0.1), control2: CGPoint(x: w * 0.35, y: h * 0.95))
                    path.addCurve(to: CGPoint(x: w, y: h * 0.55), control1: CGPoint(x: w * 0.65, y: h * 0.15), control2: CGPoint(x: w * 0.85, y: h * 0.95))
                case .pulse:
                    path.move(to: CGPoint(x: 0, y: h * 0.55))
                    path.addLine(to: CGPoint(x: w * 0.18, y: h * 0.55))
                    path.addLine(to: CGPoint(x: w * 0.3, y: h * 0.25))
                    path.addLine(to: CGPoint(x: w * 0.42, y: h * 0.78))
                    path.addLine(to: CGPoint(x: w * 0.56, y: h * 0.34))
                    path.addLine(to: CGPoint(x: w * 0.72, y: h * 0.60))
                    path.addLine(to: CGPoint(x: w, y: h * 0.55))
                case .wave:
                    path.move(to: CGPoint(x: 0, y: h * 0.55))
                    path.addCurve(to: CGPoint(x: w * 0.33, y: h * 0.55), control1: CGPoint(x: w * 0.08, y: h * 0.25), control2: CGPoint(x: w * 0.22, y: h * 0.85))
                    path.addCurve(to: CGPoint(x: w * 0.66, y: h * 0.55), control1: CGPoint(x: w * 0.44, y: h * 0.25), control2: CGPoint(x: w * 0.55, y: h * 0.85))
                    path.addCurve(to: CGPoint(x: w, y: h * 0.55), control1: CGPoint(x: w * 0.78, y: h * 0.25), control2: CGPoint(x: w * 0.90, y: h * 0.85))
                case .fractal:
                    path.move(to: CGPoint(x: 0, y: h * 0.55))
                    path.addLine(to: CGPoint(x: w * 0.12, y: h * 0.40))
                    path.addLine(to: CGPoint(x: w * 0.24, y: h * 0.74))
                    path.addLine(to: CGPoint(x: w * 0.38, y: h * 0.18))
                    path.addLine(to: CGPoint(x: w * 0.52, y: h * 0.85))
                    path.addLine(to: CGPoint(x: w * 0.67, y: h * 0.34))
                    path.addLine(to: CGPoint(x: w * 0.82, y: h * 0.62))
                    path.addLine(to: CGPoint(x: w, y: h * 0.55))
                }
            }
            .stroke(color, style: StrokeStyle(lineWidth: 1.7, lineCap: .round, lineJoin: .round))
        }
    }
}

private struct FlowLayout<Content: View>: View {
    let spacing: CGFloat
    let content: () -> Content

    init(spacing: CGFloat = 8, @ViewBuilder content: @escaping () -> Content) {
        self.spacing = spacing
        self.content = content
    }

    var body: some View {
        content()
            .frame(maxWidth: .infinity, alignment: .leading)
    }
}
