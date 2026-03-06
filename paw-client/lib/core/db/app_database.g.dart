// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'app_database.dart';

// ignore_for_file: type=lint
class $MessagesTableTable extends MessagesTable
    with TableInfo<$MessagesTableTable, MessagesTableData> {
  @override
  final GeneratedDatabase attachedDatabase;
  final String? _alias;
  $MessagesTableTable(this.attachedDatabase, [this._alias]);
  static const VerificationMeta _idMeta = const VerificationMeta('id');
  @override
  late final GeneratedColumn<String> id = GeneratedColumn<String>(
      'id', aliasedName, false,
      type: DriftSqlType.string, requiredDuringInsert: true);
  static const VerificationMeta _conversationIdMeta =
      const VerificationMeta('conversationId');
  @override
  late final GeneratedColumn<String> conversationId = GeneratedColumn<String>(
      'conversation_id', aliasedName, false,
      type: DriftSqlType.string, requiredDuringInsert: true);
  static const VerificationMeta _senderIdMeta =
      const VerificationMeta('senderId');
  @override
  late final GeneratedColumn<String> senderId = GeneratedColumn<String>(
      'sender_id', aliasedName, false,
      type: DriftSqlType.string, requiredDuringInsert: true);
  static const VerificationMeta _contentMeta =
      const VerificationMeta('content');
  @override
  late final GeneratedColumn<String> content = GeneratedColumn<String>(
      'content', aliasedName, false,
      type: DriftSqlType.string, requiredDuringInsert: true);
  static const VerificationMeta _formatMeta = const VerificationMeta('format');
  @override
  late final GeneratedColumn<String> format = GeneratedColumn<String>(
      'format', aliasedName, false,
      type: DriftSqlType.string,
      requiredDuringInsert: false,
      defaultValue: const Constant('markdown'));
  static const VerificationMeta _seqMeta = const VerificationMeta('seq');
  @override
  late final GeneratedColumn<int> seq = GeneratedColumn<int>(
      'seq', aliasedName, false,
      type: DriftSqlType.int, requiredDuringInsert: true);
  static const VerificationMeta _createdAtMeta =
      const VerificationMeta('createdAt');
  @override
  late final GeneratedColumn<DateTime> createdAt = GeneratedColumn<DateTime>(
      'created_at', aliasedName, false,
      type: DriftSqlType.dateTime, requiredDuringInsert: true);
  static const VerificationMeta _isMeMeta = const VerificationMeta('isMe');
  @override
  late final GeneratedColumn<bool> isMe = GeneratedColumn<bool>(
      'is_me', aliasedName, false,
      type: DriftSqlType.bool,
      requiredDuringInsert: false,
      defaultConstraints:
          GeneratedColumn.constraintIsAlways('CHECK ("is_me" IN (0, 1))'),
      defaultValue: const Constant(false));
  static const VerificationMeta _isAgentMeta =
      const VerificationMeta('isAgent');
  @override
  late final GeneratedColumn<bool> isAgent = GeneratedColumn<bool>(
      'is_agent', aliasedName, false,
      type: DriftSqlType.bool,
      requiredDuringInsert: false,
      defaultConstraints:
          GeneratedColumn.constraintIsAlways('CHECK ("is_agent" IN (0, 1))'),
      defaultValue: const Constant(false));
  @override
  List<GeneratedColumn> get $columns => [
        id,
        conversationId,
        senderId,
        content,
        format,
        seq,
        createdAt,
        isMe,
        isAgent
      ];
  @override
  String get aliasedName => _alias ?? actualTableName;
  @override
  String get actualTableName => $name;
  static const String $name = 'messages_table';
  @override
  VerificationContext validateIntegrity(Insertable<MessagesTableData> instance,
      {bool isInserting = false}) {
    final context = VerificationContext();
    final data = instance.toColumns(true);
    if (data.containsKey('id')) {
      context.handle(_idMeta, id.isAcceptableOrUnknown(data['id']!, _idMeta));
    } else if (isInserting) {
      context.missing(_idMeta);
    }
    if (data.containsKey('conversation_id')) {
      context.handle(
          _conversationIdMeta,
          conversationId.isAcceptableOrUnknown(
              data['conversation_id']!, _conversationIdMeta));
    } else if (isInserting) {
      context.missing(_conversationIdMeta);
    }
    if (data.containsKey('sender_id')) {
      context.handle(_senderIdMeta,
          senderId.isAcceptableOrUnknown(data['sender_id']!, _senderIdMeta));
    } else if (isInserting) {
      context.missing(_senderIdMeta);
    }
    if (data.containsKey('content')) {
      context.handle(_contentMeta,
          content.isAcceptableOrUnknown(data['content']!, _contentMeta));
    } else if (isInserting) {
      context.missing(_contentMeta);
    }
    if (data.containsKey('format')) {
      context.handle(_formatMeta,
          format.isAcceptableOrUnknown(data['format']!, _formatMeta));
    }
    if (data.containsKey('seq')) {
      context.handle(
          _seqMeta, seq.isAcceptableOrUnknown(data['seq']!, _seqMeta));
    } else if (isInserting) {
      context.missing(_seqMeta);
    }
    if (data.containsKey('created_at')) {
      context.handle(_createdAtMeta,
          createdAt.isAcceptableOrUnknown(data['created_at']!, _createdAtMeta));
    } else if (isInserting) {
      context.missing(_createdAtMeta);
    }
    if (data.containsKey('is_me')) {
      context.handle(
          _isMeMeta, isMe.isAcceptableOrUnknown(data['is_me']!, _isMeMeta));
    }
    if (data.containsKey('is_agent')) {
      context.handle(_isAgentMeta,
          isAgent.isAcceptableOrUnknown(data['is_agent']!, _isAgentMeta));
    }
    return context;
  }

  @override
  Set<GeneratedColumn> get $primaryKey => {id};
  @override
  MessagesTableData map(Map<String, dynamic> data, {String? tablePrefix}) {
    final effectivePrefix = tablePrefix != null ? '$tablePrefix.' : '';
    return MessagesTableData(
      id: attachedDatabase.typeMapping
          .read(DriftSqlType.string, data['${effectivePrefix}id'])!,
      conversationId: attachedDatabase.typeMapping.read(
          DriftSqlType.string, data['${effectivePrefix}conversation_id'])!,
      senderId: attachedDatabase.typeMapping
          .read(DriftSqlType.string, data['${effectivePrefix}sender_id'])!,
      content: attachedDatabase.typeMapping
          .read(DriftSqlType.string, data['${effectivePrefix}content'])!,
      format: attachedDatabase.typeMapping
          .read(DriftSqlType.string, data['${effectivePrefix}format'])!,
      seq: attachedDatabase.typeMapping
          .read(DriftSqlType.int, data['${effectivePrefix}seq'])!,
      createdAt: attachedDatabase.typeMapping
          .read(DriftSqlType.dateTime, data['${effectivePrefix}created_at'])!,
      isMe: attachedDatabase.typeMapping
          .read(DriftSqlType.bool, data['${effectivePrefix}is_me'])!,
      isAgent: attachedDatabase.typeMapping
          .read(DriftSqlType.bool, data['${effectivePrefix}is_agent'])!,
    );
  }

  @override
  $MessagesTableTable createAlias(String alias) {
    return $MessagesTableTable(attachedDatabase, alias);
  }
}

class MessagesTableData extends DataClass
    implements Insertable<MessagesTableData> {
  final String id;
  final String conversationId;
  final String senderId;
  final String content;
  final String format;
  final int seq;
  final DateTime createdAt;
  final bool isMe;
  final bool isAgent;
  const MessagesTableData(
      {required this.id,
      required this.conversationId,
      required this.senderId,
      required this.content,
      required this.format,
      required this.seq,
      required this.createdAt,
      required this.isMe,
      required this.isAgent});
  @override
  Map<String, Expression> toColumns(bool nullToAbsent) {
    final map = <String, Expression>{};
    map['id'] = Variable<String>(id);
    map['conversation_id'] = Variable<String>(conversationId);
    map['sender_id'] = Variable<String>(senderId);
    map['content'] = Variable<String>(content);
    map['format'] = Variable<String>(format);
    map['seq'] = Variable<int>(seq);
    map['created_at'] = Variable<DateTime>(createdAt);
    map['is_me'] = Variable<bool>(isMe);
    map['is_agent'] = Variable<bool>(isAgent);
    return map;
  }

  MessagesTableCompanion toCompanion(bool nullToAbsent) {
    return MessagesTableCompanion(
      id: Value(id),
      conversationId: Value(conversationId),
      senderId: Value(senderId),
      content: Value(content),
      format: Value(format),
      seq: Value(seq),
      createdAt: Value(createdAt),
      isMe: Value(isMe),
      isAgent: Value(isAgent),
    );
  }

  factory MessagesTableData.fromJson(Map<String, dynamic> json,
      {ValueSerializer? serializer}) {
    serializer ??= driftRuntimeOptions.defaultSerializer;
    return MessagesTableData(
      id: serializer.fromJson<String>(json['id']),
      conversationId: serializer.fromJson<String>(json['conversationId']),
      senderId: serializer.fromJson<String>(json['senderId']),
      content: serializer.fromJson<String>(json['content']),
      format: serializer.fromJson<String>(json['format']),
      seq: serializer.fromJson<int>(json['seq']),
      createdAt: serializer.fromJson<DateTime>(json['createdAt']),
      isMe: serializer.fromJson<bool>(json['isMe']),
      isAgent: serializer.fromJson<bool>(json['isAgent']),
    );
  }
  @override
  Map<String, dynamic> toJson({ValueSerializer? serializer}) {
    serializer ??= driftRuntimeOptions.defaultSerializer;
    return <String, dynamic>{
      'id': serializer.toJson<String>(id),
      'conversationId': serializer.toJson<String>(conversationId),
      'senderId': serializer.toJson<String>(senderId),
      'content': serializer.toJson<String>(content),
      'format': serializer.toJson<String>(format),
      'seq': serializer.toJson<int>(seq),
      'createdAt': serializer.toJson<DateTime>(createdAt),
      'isMe': serializer.toJson<bool>(isMe),
      'isAgent': serializer.toJson<bool>(isAgent),
    };
  }

  MessagesTableData copyWith(
          {String? id,
          String? conversationId,
          String? senderId,
          String? content,
          String? format,
          int? seq,
          DateTime? createdAt,
          bool? isMe,
          bool? isAgent}) =>
      MessagesTableData(
        id: id ?? this.id,
        conversationId: conversationId ?? this.conversationId,
        senderId: senderId ?? this.senderId,
        content: content ?? this.content,
        format: format ?? this.format,
        seq: seq ?? this.seq,
        createdAt: createdAt ?? this.createdAt,
        isMe: isMe ?? this.isMe,
        isAgent: isAgent ?? this.isAgent,
      );
  MessagesTableData copyWithCompanion(MessagesTableCompanion data) {
    return MessagesTableData(
      id: data.id.present ? data.id.value : this.id,
      conversationId: data.conversationId.present
          ? data.conversationId.value
          : this.conversationId,
      senderId: data.senderId.present ? data.senderId.value : this.senderId,
      content: data.content.present ? data.content.value : this.content,
      format: data.format.present ? data.format.value : this.format,
      seq: data.seq.present ? data.seq.value : this.seq,
      createdAt: data.createdAt.present ? data.createdAt.value : this.createdAt,
      isMe: data.isMe.present ? data.isMe.value : this.isMe,
      isAgent: data.isAgent.present ? data.isAgent.value : this.isAgent,
    );
  }

  @override
  String toString() {
    return (StringBuffer('MessagesTableData(')
          ..write('id: $id, ')
          ..write('conversationId: $conversationId, ')
          ..write('senderId: $senderId, ')
          ..write('content: $content, ')
          ..write('format: $format, ')
          ..write('seq: $seq, ')
          ..write('createdAt: $createdAt, ')
          ..write('isMe: $isMe, ')
          ..write('isAgent: $isAgent')
          ..write(')'))
        .toString();
  }

  @override
  int get hashCode => Object.hash(id, conversationId, senderId, content, format,
      seq, createdAt, isMe, isAgent);
  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      (other is MessagesTableData &&
          other.id == this.id &&
          other.conversationId == this.conversationId &&
          other.senderId == this.senderId &&
          other.content == this.content &&
          other.format == this.format &&
          other.seq == this.seq &&
          other.createdAt == this.createdAt &&
          other.isMe == this.isMe &&
          other.isAgent == this.isAgent);
}

class MessagesTableCompanion extends UpdateCompanion<MessagesTableData> {
  final Value<String> id;
  final Value<String> conversationId;
  final Value<String> senderId;
  final Value<String> content;
  final Value<String> format;
  final Value<int> seq;
  final Value<DateTime> createdAt;
  final Value<bool> isMe;
  final Value<bool> isAgent;
  final Value<int> rowid;
  const MessagesTableCompanion({
    this.id = const Value.absent(),
    this.conversationId = const Value.absent(),
    this.senderId = const Value.absent(),
    this.content = const Value.absent(),
    this.format = const Value.absent(),
    this.seq = const Value.absent(),
    this.createdAt = const Value.absent(),
    this.isMe = const Value.absent(),
    this.isAgent = const Value.absent(),
    this.rowid = const Value.absent(),
  });
  MessagesTableCompanion.insert({
    required String id,
    required String conversationId,
    required String senderId,
    required String content,
    this.format = const Value.absent(),
    required int seq,
    required DateTime createdAt,
    this.isMe = const Value.absent(),
    this.isAgent = const Value.absent(),
    this.rowid = const Value.absent(),
  })  : id = Value(id),
        conversationId = Value(conversationId),
        senderId = Value(senderId),
        content = Value(content),
        seq = Value(seq),
        createdAt = Value(createdAt);
  static Insertable<MessagesTableData> custom({
    Expression<String>? id,
    Expression<String>? conversationId,
    Expression<String>? senderId,
    Expression<String>? content,
    Expression<String>? format,
    Expression<int>? seq,
    Expression<DateTime>? createdAt,
    Expression<bool>? isMe,
    Expression<bool>? isAgent,
    Expression<int>? rowid,
  }) {
    return RawValuesInsertable({
      if (id != null) 'id': id,
      if (conversationId != null) 'conversation_id': conversationId,
      if (senderId != null) 'sender_id': senderId,
      if (content != null) 'content': content,
      if (format != null) 'format': format,
      if (seq != null) 'seq': seq,
      if (createdAt != null) 'created_at': createdAt,
      if (isMe != null) 'is_me': isMe,
      if (isAgent != null) 'is_agent': isAgent,
      if (rowid != null) 'rowid': rowid,
    });
  }

  MessagesTableCompanion copyWith(
      {Value<String>? id,
      Value<String>? conversationId,
      Value<String>? senderId,
      Value<String>? content,
      Value<String>? format,
      Value<int>? seq,
      Value<DateTime>? createdAt,
      Value<bool>? isMe,
      Value<bool>? isAgent,
      Value<int>? rowid}) {
    return MessagesTableCompanion(
      id: id ?? this.id,
      conversationId: conversationId ?? this.conversationId,
      senderId: senderId ?? this.senderId,
      content: content ?? this.content,
      format: format ?? this.format,
      seq: seq ?? this.seq,
      createdAt: createdAt ?? this.createdAt,
      isMe: isMe ?? this.isMe,
      isAgent: isAgent ?? this.isAgent,
      rowid: rowid ?? this.rowid,
    );
  }

  @override
  Map<String, Expression> toColumns(bool nullToAbsent) {
    final map = <String, Expression>{};
    if (id.present) {
      map['id'] = Variable<String>(id.value);
    }
    if (conversationId.present) {
      map['conversation_id'] = Variable<String>(conversationId.value);
    }
    if (senderId.present) {
      map['sender_id'] = Variable<String>(senderId.value);
    }
    if (content.present) {
      map['content'] = Variable<String>(content.value);
    }
    if (format.present) {
      map['format'] = Variable<String>(format.value);
    }
    if (seq.present) {
      map['seq'] = Variable<int>(seq.value);
    }
    if (createdAt.present) {
      map['created_at'] = Variable<DateTime>(createdAt.value);
    }
    if (isMe.present) {
      map['is_me'] = Variable<bool>(isMe.value);
    }
    if (isAgent.present) {
      map['is_agent'] = Variable<bool>(isAgent.value);
    }
    if (rowid.present) {
      map['rowid'] = Variable<int>(rowid.value);
    }
    return map;
  }

  @override
  String toString() {
    return (StringBuffer('MessagesTableCompanion(')
          ..write('id: $id, ')
          ..write('conversationId: $conversationId, ')
          ..write('senderId: $senderId, ')
          ..write('content: $content, ')
          ..write('format: $format, ')
          ..write('seq: $seq, ')
          ..write('createdAt: $createdAt, ')
          ..write('isMe: $isMe, ')
          ..write('isAgent: $isAgent, ')
          ..write('rowid: $rowid')
          ..write(')'))
        .toString();
  }
}

class $ConversationsTableTable extends ConversationsTable
    with TableInfo<$ConversationsTableTable, ConversationsTableData> {
  @override
  final GeneratedDatabase attachedDatabase;
  final String? _alias;
  $ConversationsTableTable(this.attachedDatabase, [this._alias]);
  static const VerificationMeta _idMeta = const VerificationMeta('id');
  @override
  late final GeneratedColumn<String> id = GeneratedColumn<String>(
      'id', aliasedName, false,
      type: DriftSqlType.string, requiredDuringInsert: true);
  static const VerificationMeta _nameMeta = const VerificationMeta('name');
  @override
  late final GeneratedColumn<String> name = GeneratedColumn<String>(
      'name', aliasedName, false,
      type: DriftSqlType.string, requiredDuringInsert: true);
  static const VerificationMeta _avatarUrlMeta =
      const VerificationMeta('avatarUrl');
  @override
  late final GeneratedColumn<String> avatarUrl = GeneratedColumn<String>(
      'avatar_url', aliasedName, true,
      type: DriftSqlType.string, requiredDuringInsert: false);
  static const VerificationMeta _lastSeqMeta =
      const VerificationMeta('lastSeq');
  @override
  late final GeneratedColumn<int> lastSeq = GeneratedColumn<int>(
      'last_seq', aliasedName, false,
      type: DriftSqlType.int,
      requiredDuringInsert: false,
      defaultValue: const Constant(0));
  static const VerificationMeta _unreadCountMeta =
      const VerificationMeta('unreadCount');
  @override
  late final GeneratedColumn<int> unreadCount = GeneratedColumn<int>(
      'unread_count', aliasedName, false,
      type: DriftSqlType.int,
      requiredDuringInsert: false,
      defaultValue: const Constant(0));
  static const VerificationMeta _updatedAtMeta =
      const VerificationMeta('updatedAt');
  @override
  late final GeneratedColumn<DateTime> updatedAt = GeneratedColumn<DateTime>(
      'updated_at', aliasedName, false,
      type: DriftSqlType.dateTime, requiredDuringInsert: true);
  @override
  List<GeneratedColumn> get $columns =>
      [id, name, avatarUrl, lastSeq, unreadCount, updatedAt];
  @override
  String get aliasedName => _alias ?? actualTableName;
  @override
  String get actualTableName => $name;
  static const String $name = 'conversations_table';
  @override
  VerificationContext validateIntegrity(
      Insertable<ConversationsTableData> instance,
      {bool isInserting = false}) {
    final context = VerificationContext();
    final data = instance.toColumns(true);
    if (data.containsKey('id')) {
      context.handle(_idMeta, id.isAcceptableOrUnknown(data['id']!, _idMeta));
    } else if (isInserting) {
      context.missing(_idMeta);
    }
    if (data.containsKey('name')) {
      context.handle(
          _nameMeta, name.isAcceptableOrUnknown(data['name']!, _nameMeta));
    } else if (isInserting) {
      context.missing(_nameMeta);
    }
    if (data.containsKey('avatar_url')) {
      context.handle(_avatarUrlMeta,
          avatarUrl.isAcceptableOrUnknown(data['avatar_url']!, _avatarUrlMeta));
    }
    if (data.containsKey('last_seq')) {
      context.handle(_lastSeqMeta,
          lastSeq.isAcceptableOrUnknown(data['last_seq']!, _lastSeqMeta));
    }
    if (data.containsKey('unread_count')) {
      context.handle(
          _unreadCountMeta,
          unreadCount.isAcceptableOrUnknown(
              data['unread_count']!, _unreadCountMeta));
    }
    if (data.containsKey('updated_at')) {
      context.handle(_updatedAtMeta,
          updatedAt.isAcceptableOrUnknown(data['updated_at']!, _updatedAtMeta));
    } else if (isInserting) {
      context.missing(_updatedAtMeta);
    }
    return context;
  }

  @override
  Set<GeneratedColumn> get $primaryKey => {id};
  @override
  ConversationsTableData map(Map<String, dynamic> data, {String? tablePrefix}) {
    final effectivePrefix = tablePrefix != null ? '$tablePrefix.' : '';
    return ConversationsTableData(
      id: attachedDatabase.typeMapping
          .read(DriftSqlType.string, data['${effectivePrefix}id'])!,
      name: attachedDatabase.typeMapping
          .read(DriftSqlType.string, data['${effectivePrefix}name'])!,
      avatarUrl: attachedDatabase.typeMapping
          .read(DriftSqlType.string, data['${effectivePrefix}avatar_url']),
      lastSeq: attachedDatabase.typeMapping
          .read(DriftSqlType.int, data['${effectivePrefix}last_seq'])!,
      unreadCount: attachedDatabase.typeMapping
          .read(DriftSqlType.int, data['${effectivePrefix}unread_count'])!,
      updatedAt: attachedDatabase.typeMapping
          .read(DriftSqlType.dateTime, data['${effectivePrefix}updated_at'])!,
    );
  }

  @override
  $ConversationsTableTable createAlias(String alias) {
    return $ConversationsTableTable(attachedDatabase, alias);
  }
}

class ConversationsTableData extends DataClass
    implements Insertable<ConversationsTableData> {
  final String id;
  final String name;
  final String? avatarUrl;
  final int lastSeq;
  final int unreadCount;
  final DateTime updatedAt;
  const ConversationsTableData(
      {required this.id,
      required this.name,
      this.avatarUrl,
      required this.lastSeq,
      required this.unreadCount,
      required this.updatedAt});
  @override
  Map<String, Expression> toColumns(bool nullToAbsent) {
    final map = <String, Expression>{};
    map['id'] = Variable<String>(id);
    map['name'] = Variable<String>(name);
    if (!nullToAbsent || avatarUrl != null) {
      map['avatar_url'] = Variable<String>(avatarUrl);
    }
    map['last_seq'] = Variable<int>(lastSeq);
    map['unread_count'] = Variable<int>(unreadCount);
    map['updated_at'] = Variable<DateTime>(updatedAt);
    return map;
  }

  ConversationsTableCompanion toCompanion(bool nullToAbsent) {
    return ConversationsTableCompanion(
      id: Value(id),
      name: Value(name),
      avatarUrl: avatarUrl == null && nullToAbsent
          ? const Value.absent()
          : Value(avatarUrl),
      lastSeq: Value(lastSeq),
      unreadCount: Value(unreadCount),
      updatedAt: Value(updatedAt),
    );
  }

  factory ConversationsTableData.fromJson(Map<String, dynamic> json,
      {ValueSerializer? serializer}) {
    serializer ??= driftRuntimeOptions.defaultSerializer;
    return ConversationsTableData(
      id: serializer.fromJson<String>(json['id']),
      name: serializer.fromJson<String>(json['name']),
      avatarUrl: serializer.fromJson<String?>(json['avatarUrl']),
      lastSeq: serializer.fromJson<int>(json['lastSeq']),
      unreadCount: serializer.fromJson<int>(json['unreadCount']),
      updatedAt: serializer.fromJson<DateTime>(json['updatedAt']),
    );
  }
  @override
  Map<String, dynamic> toJson({ValueSerializer? serializer}) {
    serializer ??= driftRuntimeOptions.defaultSerializer;
    return <String, dynamic>{
      'id': serializer.toJson<String>(id),
      'name': serializer.toJson<String>(name),
      'avatarUrl': serializer.toJson<String?>(avatarUrl),
      'lastSeq': serializer.toJson<int>(lastSeq),
      'unreadCount': serializer.toJson<int>(unreadCount),
      'updatedAt': serializer.toJson<DateTime>(updatedAt),
    };
  }

  ConversationsTableData copyWith(
          {String? id,
          String? name,
          Value<String?> avatarUrl = const Value.absent(),
          int? lastSeq,
          int? unreadCount,
          DateTime? updatedAt}) =>
      ConversationsTableData(
        id: id ?? this.id,
        name: name ?? this.name,
        avatarUrl: avatarUrl.present ? avatarUrl.value : this.avatarUrl,
        lastSeq: lastSeq ?? this.lastSeq,
        unreadCount: unreadCount ?? this.unreadCount,
        updatedAt: updatedAt ?? this.updatedAt,
      );
  ConversationsTableData copyWithCompanion(ConversationsTableCompanion data) {
    return ConversationsTableData(
      id: data.id.present ? data.id.value : this.id,
      name: data.name.present ? data.name.value : this.name,
      avatarUrl: data.avatarUrl.present ? data.avatarUrl.value : this.avatarUrl,
      lastSeq: data.lastSeq.present ? data.lastSeq.value : this.lastSeq,
      unreadCount:
          data.unreadCount.present ? data.unreadCount.value : this.unreadCount,
      updatedAt: data.updatedAt.present ? data.updatedAt.value : this.updatedAt,
    );
  }

  @override
  String toString() {
    return (StringBuffer('ConversationsTableData(')
          ..write('id: $id, ')
          ..write('name: $name, ')
          ..write('avatarUrl: $avatarUrl, ')
          ..write('lastSeq: $lastSeq, ')
          ..write('unreadCount: $unreadCount, ')
          ..write('updatedAt: $updatedAt')
          ..write(')'))
        .toString();
  }

  @override
  int get hashCode =>
      Object.hash(id, name, avatarUrl, lastSeq, unreadCount, updatedAt);
  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      (other is ConversationsTableData &&
          other.id == this.id &&
          other.name == this.name &&
          other.avatarUrl == this.avatarUrl &&
          other.lastSeq == this.lastSeq &&
          other.unreadCount == this.unreadCount &&
          other.updatedAt == this.updatedAt);
}

class ConversationsTableCompanion
    extends UpdateCompanion<ConversationsTableData> {
  final Value<String> id;
  final Value<String> name;
  final Value<String?> avatarUrl;
  final Value<int> lastSeq;
  final Value<int> unreadCount;
  final Value<DateTime> updatedAt;
  final Value<int> rowid;
  const ConversationsTableCompanion({
    this.id = const Value.absent(),
    this.name = const Value.absent(),
    this.avatarUrl = const Value.absent(),
    this.lastSeq = const Value.absent(),
    this.unreadCount = const Value.absent(),
    this.updatedAt = const Value.absent(),
    this.rowid = const Value.absent(),
  });
  ConversationsTableCompanion.insert({
    required String id,
    required String name,
    this.avatarUrl = const Value.absent(),
    this.lastSeq = const Value.absent(),
    this.unreadCount = const Value.absent(),
    required DateTime updatedAt,
    this.rowid = const Value.absent(),
  })  : id = Value(id),
        name = Value(name),
        updatedAt = Value(updatedAt);
  static Insertable<ConversationsTableData> custom({
    Expression<String>? id,
    Expression<String>? name,
    Expression<String>? avatarUrl,
    Expression<int>? lastSeq,
    Expression<int>? unreadCount,
    Expression<DateTime>? updatedAt,
    Expression<int>? rowid,
  }) {
    return RawValuesInsertable({
      if (id != null) 'id': id,
      if (name != null) 'name': name,
      if (avatarUrl != null) 'avatar_url': avatarUrl,
      if (lastSeq != null) 'last_seq': lastSeq,
      if (unreadCount != null) 'unread_count': unreadCount,
      if (updatedAt != null) 'updated_at': updatedAt,
      if (rowid != null) 'rowid': rowid,
    });
  }

  ConversationsTableCompanion copyWith(
      {Value<String>? id,
      Value<String>? name,
      Value<String?>? avatarUrl,
      Value<int>? lastSeq,
      Value<int>? unreadCount,
      Value<DateTime>? updatedAt,
      Value<int>? rowid}) {
    return ConversationsTableCompanion(
      id: id ?? this.id,
      name: name ?? this.name,
      avatarUrl: avatarUrl ?? this.avatarUrl,
      lastSeq: lastSeq ?? this.lastSeq,
      unreadCount: unreadCount ?? this.unreadCount,
      updatedAt: updatedAt ?? this.updatedAt,
      rowid: rowid ?? this.rowid,
    );
  }

  @override
  Map<String, Expression> toColumns(bool nullToAbsent) {
    final map = <String, Expression>{};
    if (id.present) {
      map['id'] = Variable<String>(id.value);
    }
    if (name.present) {
      map['name'] = Variable<String>(name.value);
    }
    if (avatarUrl.present) {
      map['avatar_url'] = Variable<String>(avatarUrl.value);
    }
    if (lastSeq.present) {
      map['last_seq'] = Variable<int>(lastSeq.value);
    }
    if (unreadCount.present) {
      map['unread_count'] = Variable<int>(unreadCount.value);
    }
    if (updatedAt.present) {
      map['updated_at'] = Variable<DateTime>(updatedAt.value);
    }
    if (rowid.present) {
      map['rowid'] = Variable<int>(rowid.value);
    }
    return map;
  }

  @override
  String toString() {
    return (StringBuffer('ConversationsTableCompanion(')
          ..write('id: $id, ')
          ..write('name: $name, ')
          ..write('avatarUrl: $avatarUrl, ')
          ..write('lastSeq: $lastSeq, ')
          ..write('unreadCount: $unreadCount, ')
          ..write('updatedAt: $updatedAt, ')
          ..write('rowid: $rowid')
          ..write(')'))
        .toString();
  }
}

abstract class _$AppDatabase extends GeneratedDatabase {
  _$AppDatabase(QueryExecutor e) : super(e);
  $AppDatabaseManager get managers => $AppDatabaseManager(this);
  late final $MessagesTableTable messagesTable = $MessagesTableTable(this);
  late final $ConversationsTableTable conversationsTable =
      $ConversationsTableTable(this);
  late final MessagesDao messagesDao = MessagesDao(this as AppDatabase);
  late final ConversationsDao conversationsDao =
      ConversationsDao(this as AppDatabase);
  @override
  Iterable<TableInfo<Table, Object?>> get allTables =>
      allSchemaEntities.whereType<TableInfo<Table, Object?>>();
  @override
  List<DatabaseSchemaEntity> get allSchemaEntities =>
      [messagesTable, conversationsTable];
}

typedef $$MessagesTableTableCreateCompanionBuilder = MessagesTableCompanion
    Function({
  required String id,
  required String conversationId,
  required String senderId,
  required String content,
  Value<String> format,
  required int seq,
  required DateTime createdAt,
  Value<bool> isMe,
  Value<bool> isAgent,
  Value<int> rowid,
});
typedef $$MessagesTableTableUpdateCompanionBuilder = MessagesTableCompanion
    Function({
  Value<String> id,
  Value<String> conversationId,
  Value<String> senderId,
  Value<String> content,
  Value<String> format,
  Value<int> seq,
  Value<DateTime> createdAt,
  Value<bool> isMe,
  Value<bool> isAgent,
  Value<int> rowid,
});

class $$MessagesTableTableFilterComposer
    extends Composer<_$AppDatabase, $MessagesTableTable> {
  $$MessagesTableTableFilterComposer({
    required super.$db,
    required super.$table,
    super.joinBuilder,
    super.$addJoinBuilderToRootComposer,
    super.$removeJoinBuilderFromRootComposer,
  });
  ColumnFilters<String> get id => $composableBuilder(
      column: $table.id, builder: (column) => ColumnFilters(column));

  ColumnFilters<String> get conversationId => $composableBuilder(
      column: $table.conversationId,
      builder: (column) => ColumnFilters(column));

  ColumnFilters<String> get senderId => $composableBuilder(
      column: $table.senderId, builder: (column) => ColumnFilters(column));

  ColumnFilters<String> get content => $composableBuilder(
      column: $table.content, builder: (column) => ColumnFilters(column));

  ColumnFilters<String> get format => $composableBuilder(
      column: $table.format, builder: (column) => ColumnFilters(column));

  ColumnFilters<int> get seq => $composableBuilder(
      column: $table.seq, builder: (column) => ColumnFilters(column));

  ColumnFilters<DateTime> get createdAt => $composableBuilder(
      column: $table.createdAt, builder: (column) => ColumnFilters(column));

  ColumnFilters<bool> get isMe => $composableBuilder(
      column: $table.isMe, builder: (column) => ColumnFilters(column));

  ColumnFilters<bool> get isAgent => $composableBuilder(
      column: $table.isAgent, builder: (column) => ColumnFilters(column));
}

class $$MessagesTableTableOrderingComposer
    extends Composer<_$AppDatabase, $MessagesTableTable> {
  $$MessagesTableTableOrderingComposer({
    required super.$db,
    required super.$table,
    super.joinBuilder,
    super.$addJoinBuilderToRootComposer,
    super.$removeJoinBuilderFromRootComposer,
  });
  ColumnOrderings<String> get id => $composableBuilder(
      column: $table.id, builder: (column) => ColumnOrderings(column));

  ColumnOrderings<String> get conversationId => $composableBuilder(
      column: $table.conversationId,
      builder: (column) => ColumnOrderings(column));

  ColumnOrderings<String> get senderId => $composableBuilder(
      column: $table.senderId, builder: (column) => ColumnOrderings(column));

  ColumnOrderings<String> get content => $composableBuilder(
      column: $table.content, builder: (column) => ColumnOrderings(column));

  ColumnOrderings<String> get format => $composableBuilder(
      column: $table.format, builder: (column) => ColumnOrderings(column));

  ColumnOrderings<int> get seq => $composableBuilder(
      column: $table.seq, builder: (column) => ColumnOrderings(column));

  ColumnOrderings<DateTime> get createdAt => $composableBuilder(
      column: $table.createdAt, builder: (column) => ColumnOrderings(column));

  ColumnOrderings<bool> get isMe => $composableBuilder(
      column: $table.isMe, builder: (column) => ColumnOrderings(column));

  ColumnOrderings<bool> get isAgent => $composableBuilder(
      column: $table.isAgent, builder: (column) => ColumnOrderings(column));
}

class $$MessagesTableTableAnnotationComposer
    extends Composer<_$AppDatabase, $MessagesTableTable> {
  $$MessagesTableTableAnnotationComposer({
    required super.$db,
    required super.$table,
    super.joinBuilder,
    super.$addJoinBuilderToRootComposer,
    super.$removeJoinBuilderFromRootComposer,
  });
  GeneratedColumn<String> get id =>
      $composableBuilder(column: $table.id, builder: (column) => column);

  GeneratedColumn<String> get conversationId => $composableBuilder(
      column: $table.conversationId, builder: (column) => column);

  GeneratedColumn<String> get senderId =>
      $composableBuilder(column: $table.senderId, builder: (column) => column);

  GeneratedColumn<String> get content =>
      $composableBuilder(column: $table.content, builder: (column) => column);

  GeneratedColumn<String> get format =>
      $composableBuilder(column: $table.format, builder: (column) => column);

  GeneratedColumn<int> get seq =>
      $composableBuilder(column: $table.seq, builder: (column) => column);

  GeneratedColumn<DateTime> get createdAt =>
      $composableBuilder(column: $table.createdAt, builder: (column) => column);

  GeneratedColumn<bool> get isMe =>
      $composableBuilder(column: $table.isMe, builder: (column) => column);

  GeneratedColumn<bool> get isAgent =>
      $composableBuilder(column: $table.isAgent, builder: (column) => column);
}

class $$MessagesTableTableTableManager extends RootTableManager<
    _$AppDatabase,
    $MessagesTableTable,
    MessagesTableData,
    $$MessagesTableTableFilterComposer,
    $$MessagesTableTableOrderingComposer,
    $$MessagesTableTableAnnotationComposer,
    $$MessagesTableTableCreateCompanionBuilder,
    $$MessagesTableTableUpdateCompanionBuilder,
    (
      MessagesTableData,
      BaseReferences<_$AppDatabase, $MessagesTableTable, MessagesTableData>
    ),
    MessagesTableData,
    PrefetchHooks Function()> {
  $$MessagesTableTableTableManager(_$AppDatabase db, $MessagesTableTable table)
      : super(TableManagerState(
          db: db,
          table: table,
          createFilteringComposer: () =>
              $$MessagesTableTableFilterComposer($db: db, $table: table),
          createOrderingComposer: () =>
              $$MessagesTableTableOrderingComposer($db: db, $table: table),
          createComputedFieldComposer: () =>
              $$MessagesTableTableAnnotationComposer($db: db, $table: table),
          updateCompanionCallback: ({
            Value<String> id = const Value.absent(),
            Value<String> conversationId = const Value.absent(),
            Value<String> senderId = const Value.absent(),
            Value<String> content = const Value.absent(),
            Value<String> format = const Value.absent(),
            Value<int> seq = const Value.absent(),
            Value<DateTime> createdAt = const Value.absent(),
            Value<bool> isMe = const Value.absent(),
            Value<bool> isAgent = const Value.absent(),
            Value<int> rowid = const Value.absent(),
          }) =>
              MessagesTableCompanion(
            id: id,
            conversationId: conversationId,
            senderId: senderId,
            content: content,
            format: format,
            seq: seq,
            createdAt: createdAt,
            isMe: isMe,
            isAgent: isAgent,
            rowid: rowid,
          ),
          createCompanionCallback: ({
            required String id,
            required String conversationId,
            required String senderId,
            required String content,
            Value<String> format = const Value.absent(),
            required int seq,
            required DateTime createdAt,
            Value<bool> isMe = const Value.absent(),
            Value<bool> isAgent = const Value.absent(),
            Value<int> rowid = const Value.absent(),
          }) =>
              MessagesTableCompanion.insert(
            id: id,
            conversationId: conversationId,
            senderId: senderId,
            content: content,
            format: format,
            seq: seq,
            createdAt: createdAt,
            isMe: isMe,
            isAgent: isAgent,
            rowid: rowid,
          ),
          withReferenceMapper: (p0) => p0
              .map((e) => (e.readTable(table), BaseReferences(db, table, e)))
              .toList(),
          prefetchHooksCallback: null,
        ));
}

typedef $$MessagesTableTableProcessedTableManager = ProcessedTableManager<
    _$AppDatabase,
    $MessagesTableTable,
    MessagesTableData,
    $$MessagesTableTableFilterComposer,
    $$MessagesTableTableOrderingComposer,
    $$MessagesTableTableAnnotationComposer,
    $$MessagesTableTableCreateCompanionBuilder,
    $$MessagesTableTableUpdateCompanionBuilder,
    (
      MessagesTableData,
      BaseReferences<_$AppDatabase, $MessagesTableTable, MessagesTableData>
    ),
    MessagesTableData,
    PrefetchHooks Function()>;
typedef $$ConversationsTableTableCreateCompanionBuilder
    = ConversationsTableCompanion Function({
  required String id,
  required String name,
  Value<String?> avatarUrl,
  Value<int> lastSeq,
  Value<int> unreadCount,
  required DateTime updatedAt,
  Value<int> rowid,
});
typedef $$ConversationsTableTableUpdateCompanionBuilder
    = ConversationsTableCompanion Function({
  Value<String> id,
  Value<String> name,
  Value<String?> avatarUrl,
  Value<int> lastSeq,
  Value<int> unreadCount,
  Value<DateTime> updatedAt,
  Value<int> rowid,
});

class $$ConversationsTableTableFilterComposer
    extends Composer<_$AppDatabase, $ConversationsTableTable> {
  $$ConversationsTableTableFilterComposer({
    required super.$db,
    required super.$table,
    super.joinBuilder,
    super.$addJoinBuilderToRootComposer,
    super.$removeJoinBuilderFromRootComposer,
  });
  ColumnFilters<String> get id => $composableBuilder(
      column: $table.id, builder: (column) => ColumnFilters(column));

  ColumnFilters<String> get name => $composableBuilder(
      column: $table.name, builder: (column) => ColumnFilters(column));

  ColumnFilters<String> get avatarUrl => $composableBuilder(
      column: $table.avatarUrl, builder: (column) => ColumnFilters(column));

  ColumnFilters<int> get lastSeq => $composableBuilder(
      column: $table.lastSeq, builder: (column) => ColumnFilters(column));

  ColumnFilters<int> get unreadCount => $composableBuilder(
      column: $table.unreadCount, builder: (column) => ColumnFilters(column));

  ColumnFilters<DateTime> get updatedAt => $composableBuilder(
      column: $table.updatedAt, builder: (column) => ColumnFilters(column));
}

class $$ConversationsTableTableOrderingComposer
    extends Composer<_$AppDatabase, $ConversationsTableTable> {
  $$ConversationsTableTableOrderingComposer({
    required super.$db,
    required super.$table,
    super.joinBuilder,
    super.$addJoinBuilderToRootComposer,
    super.$removeJoinBuilderFromRootComposer,
  });
  ColumnOrderings<String> get id => $composableBuilder(
      column: $table.id, builder: (column) => ColumnOrderings(column));

  ColumnOrderings<String> get name => $composableBuilder(
      column: $table.name, builder: (column) => ColumnOrderings(column));

  ColumnOrderings<String> get avatarUrl => $composableBuilder(
      column: $table.avatarUrl, builder: (column) => ColumnOrderings(column));

  ColumnOrderings<int> get lastSeq => $composableBuilder(
      column: $table.lastSeq, builder: (column) => ColumnOrderings(column));

  ColumnOrderings<int> get unreadCount => $composableBuilder(
      column: $table.unreadCount, builder: (column) => ColumnOrderings(column));

  ColumnOrderings<DateTime> get updatedAt => $composableBuilder(
      column: $table.updatedAt, builder: (column) => ColumnOrderings(column));
}

class $$ConversationsTableTableAnnotationComposer
    extends Composer<_$AppDatabase, $ConversationsTableTable> {
  $$ConversationsTableTableAnnotationComposer({
    required super.$db,
    required super.$table,
    super.joinBuilder,
    super.$addJoinBuilderToRootComposer,
    super.$removeJoinBuilderFromRootComposer,
  });
  GeneratedColumn<String> get id =>
      $composableBuilder(column: $table.id, builder: (column) => column);

  GeneratedColumn<String> get name =>
      $composableBuilder(column: $table.name, builder: (column) => column);

  GeneratedColumn<String> get avatarUrl =>
      $composableBuilder(column: $table.avatarUrl, builder: (column) => column);

  GeneratedColumn<int> get lastSeq =>
      $composableBuilder(column: $table.lastSeq, builder: (column) => column);

  GeneratedColumn<int> get unreadCount => $composableBuilder(
      column: $table.unreadCount, builder: (column) => column);

  GeneratedColumn<DateTime> get updatedAt =>
      $composableBuilder(column: $table.updatedAt, builder: (column) => column);
}

class $$ConversationsTableTableTableManager extends RootTableManager<
    _$AppDatabase,
    $ConversationsTableTable,
    ConversationsTableData,
    $$ConversationsTableTableFilterComposer,
    $$ConversationsTableTableOrderingComposer,
    $$ConversationsTableTableAnnotationComposer,
    $$ConversationsTableTableCreateCompanionBuilder,
    $$ConversationsTableTableUpdateCompanionBuilder,
    (
      ConversationsTableData,
      BaseReferences<_$AppDatabase, $ConversationsTableTable,
          ConversationsTableData>
    ),
    ConversationsTableData,
    PrefetchHooks Function()> {
  $$ConversationsTableTableTableManager(
      _$AppDatabase db, $ConversationsTableTable table)
      : super(TableManagerState(
          db: db,
          table: table,
          createFilteringComposer: () =>
              $$ConversationsTableTableFilterComposer($db: db, $table: table),
          createOrderingComposer: () =>
              $$ConversationsTableTableOrderingComposer($db: db, $table: table),
          createComputedFieldComposer: () =>
              $$ConversationsTableTableAnnotationComposer(
                  $db: db, $table: table),
          updateCompanionCallback: ({
            Value<String> id = const Value.absent(),
            Value<String> name = const Value.absent(),
            Value<String?> avatarUrl = const Value.absent(),
            Value<int> lastSeq = const Value.absent(),
            Value<int> unreadCount = const Value.absent(),
            Value<DateTime> updatedAt = const Value.absent(),
            Value<int> rowid = const Value.absent(),
          }) =>
              ConversationsTableCompanion(
            id: id,
            name: name,
            avatarUrl: avatarUrl,
            lastSeq: lastSeq,
            unreadCount: unreadCount,
            updatedAt: updatedAt,
            rowid: rowid,
          ),
          createCompanionCallback: ({
            required String id,
            required String name,
            Value<String?> avatarUrl = const Value.absent(),
            Value<int> lastSeq = const Value.absent(),
            Value<int> unreadCount = const Value.absent(),
            required DateTime updatedAt,
            Value<int> rowid = const Value.absent(),
          }) =>
              ConversationsTableCompanion.insert(
            id: id,
            name: name,
            avatarUrl: avatarUrl,
            lastSeq: lastSeq,
            unreadCount: unreadCount,
            updatedAt: updatedAt,
            rowid: rowid,
          ),
          withReferenceMapper: (p0) => p0
              .map((e) => (e.readTable(table), BaseReferences(db, table, e)))
              .toList(),
          prefetchHooksCallback: null,
        ));
}

typedef $$ConversationsTableTableProcessedTableManager = ProcessedTableManager<
    _$AppDatabase,
    $ConversationsTableTable,
    ConversationsTableData,
    $$ConversationsTableTableFilterComposer,
    $$ConversationsTableTableOrderingComposer,
    $$ConversationsTableTableAnnotationComposer,
    $$ConversationsTableTableCreateCompanionBuilder,
    $$ConversationsTableTableUpdateCompanionBuilder,
    (
      ConversationsTableData,
      BaseReferences<_$AppDatabase, $ConversationsTableTable,
          ConversationsTableData>
    ),
    ConversationsTableData,
    PrefetchHooks Function()>;

class $AppDatabaseManager {
  final _$AppDatabase _db;
  $AppDatabaseManager(this._db);
  $$MessagesTableTableTableManager get messagesTable =>
      $$MessagesTableTableTableManager(_db, _db.messagesTable);
  $$ConversationsTableTableTableManager get conversationsTable =>
      $$ConversationsTableTableTableManager(_db, _db.conversationsTable);
}
