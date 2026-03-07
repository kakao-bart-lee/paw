import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:get_it/get_it.dart';

import '../../../core/errors/app_error.dart';
import '../../../core/http/api_client.dart';

class ProfileState {
  final AsyncValue<Map<String, dynamic>> userAsync;
  final bool isSaving;

  const ProfileState({
    this.userAsync = const AsyncValue.loading(),
    this.isSaving = false,
  });

  ProfileState copyWith({
    AsyncValue<Map<String, dynamic>>? userAsync,
    bool? isSaving,
  }) {
    return ProfileState(
      userAsync: userAsync ?? this.userAsync,
      isSaving: isSaving ?? this.isSaving,
    );
  }
}

class ProfileNotifier extends Notifier<ProfileState> {
  ApiClient get _apiClient => GetIt.instance<ApiClient>();

  @override
  ProfileState build() {
    return const ProfileState();
  }

  Future<void> loadProfile() async {
    if (_apiClient.accessToken == null) {
      state = state.copyWith(
        userAsync: const AsyncValue.data(<String, dynamic>{}),
      );
      return;
    }

    state = state.copyWith(userAsync: const AsyncValue.loading());
    try {
      final result = await _apiClient.getMe();
      state = state.copyWith(userAsync: AsyncValue.data(result));
    } catch (e, st) {
      final uiError = AppErrorMapper.map(e);
      state = state.copyWith(
        userAsync: AsyncValue.error(Exception(uiError.message), st),
      );
    }
  }

  Future<void> updateProfile(String displayName) async {
    if (_apiClient.accessToken == null) {
      state = state.copyWith(
        userAsync: AsyncValue.error(
          Exception('Authentication required'),
          StackTrace.current,
        ),
      );
      return;
    }

    state = state.copyWith(isSaving: true);
    try {
      await _apiClient.updateMe(displayName: displayName);
      await loadProfile();
    } catch (e, st) {
      final uiError = AppErrorMapper.map(e);
      state = state.copyWith(
        userAsync: AsyncValue.error(Exception(uiError.message), st),
        isSaving: false,
      );
      return;
    }
    state = state.copyWith(isSaving: false);
  }
}

final profileProvider = NotifierProvider<ProfileNotifier, ProfileState>(
  ProfileNotifier.new,
);
