// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'typing_provider.dart';

// **************************************************************************
// RiverpodGenerator
// **************************************************************************

// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint, type=warning

@ProviderFor(TypingNotifier)
final typingProvider = TypingNotifierProvider._();

final class TypingNotifierProvider
    extends $NotifierProvider<TypingNotifier, Map<String, Set<String>>> {
  TypingNotifierProvider._()
      : super(
          from: null,
          argument: null,
          retry: null,
          name: r'typingProvider',
          isAutoDispose: true,
          dependencies: null,
          $allTransitiveDependencies: null,
        );

  @override
  String debugGetCreateSourceHash() => _$typingNotifierHash();

  @$internal
  @override
  TypingNotifier create() => TypingNotifier();

  /// {@macro riverpod.override_with_value}
  Override overrideWithValue(Map<String, Set<String>> value) {
    return $ProviderOverride(
      origin: this,
      providerOverride: $SyncValueProvider<Map<String, Set<String>>>(value),
    );
  }
}

String _$typingNotifierHash() => r'ec49abb1c38767717de4b0902118e5b7533ccee1';

abstract class _$TypingNotifier extends $Notifier<Map<String, Set<String>>> {
  Map<String, Set<String>> build();
  @$mustCallSuper
  @override
  void runBuild() {
    final ref =
        this.ref as $Ref<Map<String, Set<String>>, Map<String, Set<String>>>;
    final element = ref.element as $ClassProviderElement<
        AnyNotifier<Map<String, Set<String>>, Map<String, Set<String>>>,
        Map<String, Set<String>>,
        Object?,
        Object?>;
    element.handleCreate(ref, build);
  }
}
